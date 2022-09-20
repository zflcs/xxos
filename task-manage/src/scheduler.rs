
/// Schedule trait，根据进程的 id 进行调度
/// add 添加 id
/// fetch 取出 id
pub trait Schedule<I: Copy + Ord>: 'static {
    fn add(&mut self, id: I);
    fn fetch(&mut self) -> I;
}