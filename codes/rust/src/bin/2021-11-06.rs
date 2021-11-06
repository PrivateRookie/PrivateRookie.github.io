use std::cell::RefCell;

fn main() {
    let x = RefCell::new(vec![1, 2, 3, 4]);
    // compile ok
    if let Some(idx) = x.borrow().iter().position(|&i| i == 3) {
        // x immutable borrow live till the end of this scope
        println!("found {}", idx);
        println!("total len {}", x.borrow().len());
        // panic on runtime
        x.borrow_mut().pop();
    };
    let may_found = x.borrow().iter().position(|&i| i == 3);
    if let Some(idx) = may_found {
        println!("found {}", idx);
        println!("total len {}", x.borrow().len());
        x.borrow_mut().pop();
    }
}
