
/// Manager trait
pub trait Schedule: Send + Sync {
    /// 任务对象
    type Item;
    /// 删除 item
    fn add(&mut self, task: Self::Item);
    /// 获取 mut item
    fn fetch(&mut self) -> Option<Self::Item>;
}
