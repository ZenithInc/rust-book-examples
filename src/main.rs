struct User;

fn song(_user: &User) {}

fn dancing(_user: &User) {}

fn main() {
    let user = User{};
    song(&user);   // 所有权已经转移
    dancing(&user);  // 编译错误
}