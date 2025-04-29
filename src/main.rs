trait Animal {}

struct Dog;
impl Animal for Dog {}

// 这个函数可以处理任何实现了 Animal trait 的类型的引用
fn feed_animal(_: &dyn Animal) {
    println!("Feeding an animal");
}

// 这个函数只接受 Dog 类型的引用
fn feed_dog(d: &Dog) {
    println!("Feeding a dog");
    feed_animal(d);
}

// 定义一个函数指针类型，它接受 &dyn Animal
type AnimalFeeder = fn(&dyn Animal);

//定义一个函数指针类型，它接受 &Dog
type DogFeeder = fn(&Dog);

fn main() {
    // 一个 AnimalFeeder 类型的变量
    let animal_feeder: AnimalFeeder = feed_animal;

    // 一个 DogFeeder 类型的变量
    let dog_feeder: DogFeeder = feed_dog;

    // 下面这行是合法的！
    let animal_feeder2: AnimalFeeder = feed_dog;

    // 下面这行是不合法的！编译不通过！
    // let dog_feeder2: DogFeeder = feed_animal;

    //现在，`animal_feeder2` 可以处理 `&dyn Animal` 也可以处理 `&Dog`
    //虽然其定义的类型为 AnimalFeeder，但是其指向的函数指针 feed_dog, 可以处理 &Dog类型的入参。
    let dog = Dog;
    animal_feeder2(&dog);
    let animal: &dyn Animal = &dog;
    animal_feeder2(animal);

    //正常调用
    animal_feeder(animal);
    dog_feeder(&dog);
}
