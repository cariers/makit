//! 将手牌与仍留存的弃牌组合为已经提交的固定组。
//!
//! 这里只检查持有数量、固定组引用与声明结构；响应窗口、牌形、通配解释及补牌流程由上层管理。

use makit_core::{DiscardId, Melds, RiverError, Seat, Tile};

use crate::{Context, ContextError, Variant};

impl<V: Variant> Context<V> {
    /// 消耗本家两张手牌与仍留存的他家弃牌，追加吃组并返回其零基索引。
    ///
    /// 鸣取牌及来源直接读取当前牌河记录；弃牌被取走后，其历史和编号仍然保留。
    /// 本家牌面沿用 [`makit_core::Hand::take_tiles`] 按原排列匹配，优先消费旧牌区，
    /// 重复牌面按请求数量扣除。剩余手牌保留分区，不自动归并摸牌区。
    ///
    /// 请求牌序保留在组内，鸣取牌位于最后。本方法提交已裁定的声明，不检查是否
    /// 轮到该座位、是否来自上家、顺子牌形或通配解释，也不处理竞争响应或后续流程。
    ///
    /// # Errors
    ///
    /// 本家或弃牌来源尚未加入时返回 [`ContextError::MissingSeat`]；鸣取自身弃牌时
    /// 返回 [`ContextError::OwnDiscard`]；弃牌不存在或已被取走时返回
    /// [`ContextError::River`]；手牌中请求牌面的数量不足时返回 [`ContextError::Hand`]。
    /// 返回错误时，手牌、牌河及固定组列表均保持不变。
    pub fn chow(
        &mut self,
        seat: Seat,
        id: DiscardId,
        tiles: [Tile; 2],
    ) -> Result<usize, ContextError> {
        self.commit_claim(seat, id, tiles, Melds::chow)
    }

    /// 消耗本家两张手牌与仍留存的他家弃牌，追加碰组并返回其零基索引。
    ///
    /// 鸣取牌及来源读取当前牌河记录，消费后保留弃牌历史与编号。本家牌面按手牌
    /// 原排列匹配，优先消费旧牌区，重复牌面按请求数量扣除；剩余手牌保留分区。
    /// 请求牌序保留在组内，鸣取牌位于最后。
    ///
    /// 本方法提交已裁定的碰牌，不校验同牌关系、通配解释或行动权限，
    /// 不处理竞争响应，也不自动推进后续流程。
    ///
    /// # Errors
    ///
    /// 本家或弃牌来源尚未加入时返回 [`ContextError::MissingSeat`]；鸣取自身弃牌时
    /// 返回 [`ContextError::OwnDiscard`]；弃牌不存在或已被取走时返回
    /// [`ContextError::River`]；手牌中请求牌面的数量不足时返回 [`ContextError::Hand`]。
    /// 返回错误时，手牌、牌河及固定组列表均保持不变。
    pub fn pong(
        &mut self,
        seat: Seat,
        id: DiscardId,
        tiles: [Tile; 2],
    ) -> Result<usize, ContextError> {
        self.commit_claim(seat, id, tiles, Melds::pong)
    }

    /// 消耗本家三张手牌与仍留存的他家弃牌，追加直接明杠并返回其零基索引。
    ///
    /// 鸣取牌及来源读取当前牌河记录，消费后保留弃牌历史与编号。本家牌面按手牌
    /// 原排列匹配，优先消费旧牌区，重复牌面按请求数量扣除；剩余手牌保留分区。
    /// 请求牌序保留在组内，鸣取牌位于最后。
    ///
    /// 本方法提交已裁定的明杠，不校验同牌关系、通配解释或行动权限，
    /// 不裁定竞争响应或抢杠窗口，也不会自动补牌。
    ///
    /// # Errors
    ///
    /// 本家或弃牌来源尚未加入时返回 [`ContextError::MissingSeat`]；鸣取自身弃牌时
    /// 返回 [`ContextError::OwnDiscard`]；弃牌不存在或已被取走时返回
    /// [`ContextError::River`]；手牌中请求牌面的数量不足时返回 [`ContextError::Hand`]。
    /// 返回错误时，手牌、牌河及固定组列表均保持不变。
    pub fn claimed_kong(
        &mut self,
        seat: Seat,
        id: DiscardId,
        tiles: [Tile; 3],
    ) -> Result<usize, ContextError> {
        self.commit_claim(seat, id, tiles, Melds::claimed_kong)
    }

    /// 从本家手牌取出四张牌，追加暗杠并返回固定组的零基索引。
    ///
    /// 按输入的四张原始牌面消费手牌，重复牌面表示请求多张副本；匹配时优先消费
    /// 原排列中的旧牌。请求顺序保留在暗杠中，剩余手牌保留原分区。
    /// 本方法只提交已裁定的暗杠，不检查牌形、通配解释、声明权限或抢杠窗口，
    /// 也不会自动补牌。
    ///
    /// # Errors
    ///
    /// 座位尚未加入时返回 [`ContextError::MissingSeat`]；手牌中请求牌面的数量不足时
    /// 返回 [`ContextError::Hand`]。返回错误时，手牌及固定组列表均保持不变。
    pub fn concealed_kong(&mut self, seat: Seat, tiles: [Tile; 4]) -> Result<usize, ContextError> {
        let seat_context = self
            .seats
            .get(seat)
            .ok_or(ContextError::MissingSeat { seat })?;
        let mut hand = seat_context.hand.clone();
        let tiles = hand
            .take_tiles(tiles)
            .map_err(|source| ContextError::Hand { seat, source })?;
        let mut melds = seat_context.melds.clone();
        let index = melds.concealed_kong(tiles);

        let seat_context = self
            .seats
            .get_mut(seat)
            .expect("validated seat must remain joined until commit");
        seat_context.hand = hand;
        seat_context.melds = melds;
        Ok(index)
    }

    /// 从本家手牌取出一张牌，将指定碰组原位替换为加杠。
    ///
    /// `meld_index` 指向本家当前固定组列表，`tile` 指定从手牌消费的原始牌面。
    /// 匹配时选取原排列中最早的副本，优先消费旧牌区。
    /// 成功后固定组数量和索引不变，原碰牌的三张牌、顺序及鸣取来源完整保留，
    /// 新增牌位于第四张；剩余手牌保留原分区。
    ///
    /// 本方法表示加杠最终提交，不创建或裁定抢杠窗口，不检查牌形和通配解释，
    /// 也不会自动补牌。需要响应交互的玩法应先在上层完成裁决，再调用本方法。
    ///
    /// # Errors
    ///
    /// 座位尚未加入时返回 [`ContextError::MissingSeat`]；固定组索引越界或目标不是碰组时
    /// 返回 [`ContextError::Melds`]；手牌中没有请求牌面时返回 [`ContextError::Hand`]。
    /// 返回错误时，手牌及原固定组均保持不变。
    pub fn add_kong(
        &mut self,
        seat: Seat,
        meld_index: usize,
        tile: Tile,
    ) -> Result<(), ContextError> {
        let seat_context = self
            .seats
            .get(seat)
            .ok_or(ContextError::MissingSeat { seat })?;
        let mut melds = seat_context.melds.clone();
        melds
            .add_kong(meld_index, tile)
            .map_err(|source| ContextError::Melds { seat, source })?;
        let mut hand = seat_context.hand.clone();
        hand.take_tiles([tile])
            .map_err(|source| ContextError::Hand { seat, source })?;

        let seat_context = self
            .seats
            .get_mut(seat)
            .expect("validated seat must remain joined until commit");
        seat_context.hand = hand;
        seat_context.melds = melds;
        Ok(())
    }

    /// 共用已裁定鸣牌的来源验证与原子提交，只接收本模块指定的固定组追加方法。
    fn commit_claim<const N: usize>(
        &mut self,
        seat: Seat,
        id: DiscardId,
        tiles: [Tile; N],
        append: fn(&mut Melds, [Tile; N], Tile, Seat) -> usize,
    ) -> Result<usize, ContextError> {
        let seat_context = self
            .seats
            .get(seat)
            .ok_or(ContextError::MissingSeat { seat })?;
        let discard = self.river.get(id).ok_or(ContextError::River {
            source: RiverError::UnknownDiscard { id },
        })?;
        let source_seat = discard.seat();
        if self.seats.get(source_seat).is_none() {
            return Err(ContextError::MissingSeat { seat: source_seat });
        }
        if source_seat == seat {
            return Err(ContextError::OwnDiscard { seat, id });
        }

        // 只复制参与本次转移的核心状态，避免为变体扩展增加 Clone 约束。
        let mut hand = seat_context.hand.clone();
        let mut river = self.river.clone();
        let claimed = river
            .take(id)
            .map_err(|source| ContextError::River { source })?;
        let owned = hand
            .take_tiles(tiles)
            .map_err(|source| ContextError::Hand { seat, source })?;
        let mut melds = seat_context.melds.clone();
        let index = append(&mut melds, owned, claimed, source_seat);

        // 全部可失败操作均已在候选上完成；期间未改变已验证的座位成员集合。
        let seat_context = self
            .seats
            .get_mut(seat)
            .expect("validated seat must remain joined until commit");
        seat_context.hand = hand;
        seat_context.melds = melds;
        self.river = river;
        Ok(index)
    }
}
