//! 牌墙、手牌与牌河之间，以及多个座位手牌之间的原子转移。

use makit_core::{Discard, DiscardId, Hand, Seat, SeatMap, Tile, Wall, WallError};

use super::Context;
use crate::{ContextError, Variant};

/// 牌墙当前剩余区间的逻辑取牌端，不表示桌面方位或补牌规则。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WallEnd {
    /// 从当前前端按牌序取牌。
    Front,
    /// 从当前后端逆牌序取牌。
    Back,
}

/// 发牌批次中的一项请求，批次按请求顺序消费牌墙。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DealRequest {
    /// 接收本项牌的已加入座位。
    pub seat: Seat,
    /// 本项使用的逻辑取牌端。
    pub end: WallEnd,
    /// 本项发放张数，允许为零。
    pub count: usize,
}

/// 一项已提交发牌的结果，与对应请求保持相同的座位和取牌端。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DealOutcome {
    /// 实际收牌座位。
    pub seat: Seat,
    /// 实际使用的逻辑取牌端。
    pub end: WallEnd,
    /// 按实际取牌顺序排列的原始牌面，已加入该座位的旧牌区。
    pub tiles: Vec<Tile>,
}

/// 换牌批次中的一条有向转移，张数和方向均由调用者决定。
///
/// 原始牌面与重复次数共同表达选择，同牌面的副本不作身份区分。
/// 不同条目可以使用同一来源，其移出总量以整批提交前的来源手牌校验。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExchangeSelection {
    /// 移出牌的已加入座位。
    pub from: Seat,
    /// 接收牌的另一已加入座位。
    pub to: Seat,
    /// 要移出的原始牌面，重复项表示多张，顺序决定本项转移的牌序。
    pub tiles: Vec<Tile>,
}

/// 一条已提交换牌的结果，不表示行动开始、等待输入或客户端公开事件。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExchangeTransfer {
    /// 实际移出牌的座位。
    pub from: Seat,
    /// 实际收到牌的座位。
    pub to: Seat,
    /// 按对应选择顺序排列的原始牌面，已加入目标的旧牌区。
    pub tiles: Vec<Tile>,
}

impl<V: Variant> Context<V> {
    /// 从指定牌墙端摸入多张牌，按取牌顺序追加到指定座位的摸牌区。
    ///
    /// 保留原有旧牌区和摸牌区，不要求已经指定庄家或准备其他座位。
    /// 取零张仍检查座位是否存在；成功后返回空向量且不修改任何状态。
    ///
    /// # Errors
    ///
    /// 座位未加入时返回 [`ContextError::MissingSeat`]；牌墙不足时返回
    /// [`ContextError::Wall`]。返回错误时牌墙与手牌均保持不变。
    pub fn draw(
        &mut self,
        seat: Seat,
        end: WallEnd,
        count: usize,
    ) -> Result<Vec<Tile>, ContextError> {
        let state = self
            .seats
            .get(seat)
            .ok_or(ContextError::MissingSeat { seat })?;
        if count == 0 {
            return Ok(Vec::new());
        }

        let mut wall = self.wall.clone();
        let mut hand = state.hand.clone();
        let tiles = draw_tiles(&mut wall, end, count)?;
        hand.draw_many(tiles.iter().copied());

        // 候选只包含核心容器，变体扩展无需复制；完成收牌后才提交两侧状态。
        self.seats
            .get_mut(seat)
            .expect("a validated draw seat must remain joined")
            .hand = hand;
        self.wall = wall;
        Ok(tiles)
    }

    /// 按请求顺序原子发牌，将每项收到的牌插入目标座位的旧牌区末尾。
    ///
    /// 同一座位可以出现多次，各项可以选择不同的牌墙端；不自动按座位排序，
    /// 也不固定轮次或发放张数。已有摸牌区保留其来源语义。
    /// 返回项与请求一一对应；空批次成功且不修改状态，零张请求仍检查座位。
    ///
    /// # Errors
    ///
    /// 任一座位未加入时返回 [`ContextError::MissingSeat`]；执行某项时剩余牌不足
    /// 返回 [`ContextError::Wall`]，其中剩余量为该项执行前的候选牌墙数量。
    /// 无论哪一项失败，原牌墙及全部手牌均保持不变。
    pub fn deal(&mut self, requests: &[DealRequest]) -> Result<Vec<DealOutcome>, ContextError> {
        for request in requests {
            if !self.seats.contains_key(request.seat) {
                return Err(ContextError::MissingSeat { seat: request.seat });
            }
        }
        if requests.iter().all(|request| request.count == 0) {
            return Ok(requests
                .iter()
                .map(|request| DealOutcome {
                    seat: request.seat,
                    end: request.end,
                    tiles: Vec::new(),
                })
                .collect());
        }

        let mut wall = self.wall.clone();
        let mut hands: SeatMap<Hand> = SeatMap::new();
        let mut outcomes = Vec::with_capacity(requests.len());
        for request in requests {
            let tiles = draw_tiles(&mut wall, request.end, request.count)?;
            if !tiles.is_empty() {
                let hand = candidate_hand(&self.seats, &mut hands, request.seat);
                hand.receive(tiles.iter().copied());
            }
            outcomes.push(DealOutcome {
                seat: request.seat,
                end: request.end,
                tiles,
            });
        }

        commit_hands(self, hands);
        self.wall = wall;
        Ok(outcomes)
    }

    /// 打出指定座位的一张牌并追加牌河历史，返回新弃牌编号。
    ///
    /// 匹配手牌中最早出现的同牌面副本，优先使用旧牌区；保留移出前的摸牌来源，
    /// 成功后将余牌全部归入旧牌区，不进行通配替换。
    ///
    /// # Errors
    ///
    /// 座位未加入时返回 [`ContextError::MissingSeat`]；指定牌面不存在时返回
    /// [`ContextError::Hand`]；无法分配弃牌编号时返回 [`ContextError::River`]。
    /// 返回错误时手牌、分区及牌河历史均保持不变。
    pub fn discard(&mut self, seat: Seat, tile: Tile) -> Result<DiscardId, ContextError> {
        let mut ids = self.discard_many(seat, &[tile])?;
        // 每张成功打出的牌对应一个追加编号，单张请求的结果必定非空。
        Ok(ids
            .pop()
            .expect("a successful single discard must return one discard id"))
    }

    /// 原子打出指定座位的多张原始牌面，按请求顺序追加牌河历史。
    ///
    /// 重复牌面表示请求多个副本，每项匹配尚未选中的最早副本，优先使用旧牌区。
    /// 全部摸牌来源以操作前的同一分区为准；非空批次成功后才将余牌全部归入旧牌区。
    /// 空列表仍检查座位，成功后不改变分区或牌河；返回编号与请求牌面一一对应。
    /// 不执行通配替换，也不要求牌墙仍有剩余牌。
    ///
    /// # Errors
    ///
    /// 座位未加入时返回 [`ContextError::MissingSeat`]；任一牌面的副本不足时返回
    /// [`ContextError::Hand`]；无法为整批分配弃牌编号时返回 [`ContextError::River`]。
    /// 任一错误均不修改原手牌、分区或牌河。
    pub fn discard_many(
        &mut self,
        seat: Seat,
        tiles: &[Tile],
    ) -> Result<Vec<DiscardId>, ContextError> {
        let state = self
            .seats
            .get(seat)
            .ok_or(ContextError::MissingSeat { seat })?;
        if tiles.is_empty() {
            return Ok(Vec::new());
        }

        let mut hand = state.hand.clone();
        let discards = hand
            .discard_tiles(tiles)
            .map_err(|source| ContextError::Hand { seat, source })?
            .into_iter()
            .map(|discard| Discard::new(seat, discard))
            .collect();
        // 手牌扣除成功后，牌河仍可能因编号容量失败，因此二者都在候选上执行。
        let mut river = self.river.clone();
        let ids = river
            .append_many(discards)
            .map_err(|source| ContextError::River { source })?;

        self.seats
            .get_mut(seat)
            .expect("a validated discard seat must remain joined")
            .hand = hand;
        self.river = river;
        Ok(ids)
    }

    /// 原子提交多个座位之间的换牌，不固定张数、方向或交换人数。
    ///
    /// 重复牌面表示数量，不标识某个副本。同一来源可以向多个目标移出，移出前
    /// 汇总该来源整批请求；每张牌匹配尚未选中的最早副本，优先使用旧牌区。
    /// 所有来源均完成移出后，才按请求顺序收牌，同批收到的牌不能用于满足移出请求。
    /// 余牌保留原分区，收到的牌按各项选择顺序插入目标旧牌区末尾，不执行通配替换。
    ///
    /// 返回项与请求一一对应；空批次成功且不修改状态，空选择仍校验两端座位。
    /// 调用者负责确认提交前状态版本以及玩法是否允许这些选择。
    ///
    /// # Errors
    ///
    /// 任一端点未加入时返回 [`ContextError::MissingSeat`]；来源与目标相同时返回
    /// [`ContextError::SelfTransfer`]；任一来源的原手牌不足以满足整批请求时返回
    /// [`ContextError::Hand`]，其中牌面所需数量与可用数量均按该来源整批计算。
    /// 任一错误均不修改任何座位的原手牌或分区。
    pub fn exchange(
        &mut self,
        selections: &[ExchangeSelection],
    ) -> Result<Vec<ExchangeTransfer>, ContextError> {
        let mut removals: SeatMap<Vec<Tile>> = SeatMap::new();
        for selection in selections {
            for seat in [selection.from, selection.to] {
                if !self.seats.contains_key(seat) {
                    return Err(ContextError::MissingSeat { seat });
                }
            }
            if selection.from == selection.to {
                return Err(ContextError::SelfTransfer {
                    seat: selection.from,
                });
            }
            removals[selection.from]
                .get_or_insert_with(Vec::new)
                .extend_from_slice(&selection.tiles);
        }

        let mut hands = SeatMap::new();
        for (seat, tiles) in removals.iter() {
            if !tiles.is_empty() {
                let hand = candidate_hand(&self.seats, &mut hands, seat);
                // 每个来源整批扣除一次，保证不足错误描述总请求量和原持有量。
                hand.take_tiles_many(tiles)
                    .map_err(|source| ContextError::Hand { seat, source })?;
            }
        }

        let mut transfers = Vec::with_capacity(selections.len());
        for selection in selections {
            // 所有原始牌面和副本数量已经实际扣除，可按请求恢复各条转移的牌序。
            let tiles = selection.tiles.clone();
            if !tiles.is_empty() {
                let hand = candidate_hand(&self.seats, &mut hands, selection.to);
                hand.receive(tiles.iter().copied());
            }
            transfers.push(ExchangeTransfer {
                from: selection.from,
                to: selection.to,
                tiles,
            });
        }

        commit_hands(self, hands);
        Ok(transfers)
    }
}

/// 将逻辑端映射到底层受检取牌，不附加玩法含义。
fn draw_tiles(wall: &mut Wall, end: WallEnd, count: usize) -> Result<Vec<Tile>, ContextError> {
    let result: Result<Vec<Tile>, WallError> = match end {
        WallEnd::Front => wall.draw_front_many(count),
        WallEnd::Back => wall.draw_back_many(count),
    };
    result.map_err(|source| ContextError::Wall { source })
}

/// 首次影响某座位时才复制其手牌，不复制固定组或变体状态。
fn candidate_hand<'a, V: Variant>(
    seats: &SeatMap<super::SeatContext<V>>,
    hands: &'a mut SeatMap<Hand>,
    seat: Seat,
) -> &'a mut Hand {
    hands[seat].get_or_insert_with(|| {
        seats
            .get(seat)
            .expect("a validated candidate seat must remain joined")
            .hand
            .clone()
    })
}

/// 只在全部可失败操作完成后替换候选手牌，保留座位成员和各自扩展。
fn commit_hands<V: Variant>(context: &mut Context<V>, mut hands: SeatMap<Hand>) {
    for (seat, state) in context.seats.iter_mut() {
        if let Some(hand) = hands.remove(seat) {
            state.hand = hand;
        }
    }
}
