
extern crate utils;
use utils::Downcast;
use core::any::Any;
use task_manage::Task;

#[derive(Downcast)]
struct StructName1(pub usize);


fn main() {
    let a: Box<dyn Task> = Box::new(StructName1(15654));
    let task1: &StructName1 = StructName1::downcast(a.as_ref());
    println!("{}", task1.0);
    task1.execute();
}
