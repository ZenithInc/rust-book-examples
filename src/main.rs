struct Node {
    value: i32,
    next: Option<Box<Node>>,
}

struct List {
    head: Option<Box<Node>>,
}

impl List {
    fn new() -> Self {
        List { head: None }
    }

    fn push(&mut self, value: i32) {
        let node = Node {
            value,
            next: self.head.take(), // take 是将原来的 head 置空并取出旧值
        };
        self.head = Some(Box::new(node));
    }
}

fn main() {
    let mut list = List::new();
    list.push(1);
    list.push(2);
}