use project23::stack::Stack;

fn main() {
    let vec = vec![1, 2, 3, 4, 5];
    let stack: Stack<_> = vec.iter().map(|x| x.to_string()).collect();
    let mut stack: Stack<_> = [1, 2, 3, 4, 5].into();
    stack.push_slice(&[6, 7, 8, 9, 10]);
    while let Some(top) = stack.pop() {
        println!("{}", top);
    }
    println!("Stack size: {}", stack.size());
}

#[cfg(test)]
mod test;
