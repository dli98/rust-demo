// Rust Trait 学习示例

use std::ops::Add;
use std::fmt::Display;

// 1. 基础 trait 实现
#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Point({}, {})", self.x, self.y)
    }
}

// 2. 自定义 trait
trait Drawable {
    fn draw(&self);
    fn area(&self) -> f64;

    // 默认实现
    fn describe(&self) {
        println!("这是一个可绘制的图形，面积为: {}", self.area());
    }
}

impl Drawable for Point {
    fn draw(&self) {
        println!("绘制点: {}", self);
    }

    fn area(&self) -> f64 {
        0.0 // 点的面积为0
    }
}

// 3. 更复杂的图形
#[derive(Debug)]
struct Circle {
    center: Point,
    radius: f64,
}

impl Circle {
    fn new(center: Point, radius: f64) -> Self {
        Circle { center, radius }
    }
}

impl Drawable for Circle {
    fn draw(&self) {
        println!("绘制圆形: 中心{}, 半径{}", self.center, self.radius);
    }

    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

impl Display for Circle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Circle(center: {}, radius: {})", self.center, self.radius)
    }
}

// 4. 泛型 trait
trait Comparable<T> {
    fn compare(&self, other: &T) -> std::cmp::Ordering;
}

impl Comparable<Point> for Point {
    fn compare(&self, other: &Point) -> std::cmp::Ordering {
        let self_distance = (self.x * self.x + self.y * self.y).sqrt();
        let other_distance = (other.x * other.x + other.y * other.y).sqrt();
        self_distance.partial_cmp(&other_distance).unwrap_or(std::cmp::Ordering::Equal)
    }
}

// 5. trait 作为参数 trait 对象版本（动态分发）
fn draw_shape(shape: &dyn Drawable) {
    shape.draw();
    shape.describe();
}

// 静态分发（编译时确定）- 泛型
fn draw_shape_gen<T:Drawable>(shape: &T){
    shape.draw();
    shape.describe();
}

// 6. trait bound
fn compare_and_draw<T>(item1: &T, item2: &T)
where
    T: Drawable + Display + Comparable<T>
{
    println!("比较两个图形:");
    println!("图形1: {}", item1);
    println!("图形2: {}", item2);

    match item1.compare(item2) {
        std::cmp::Ordering::Less => println!("图形1 < 图形2"),
        std::cmp::Ordering::Greater => println!("图形1 > 图形2"),
        std::cmp::Ordering::Equal => println!("图形1 = 图形2"),
    }

    item1.draw();
    item2.draw();
}


fn main() {
    println!("=== Rust Trait 学习示例 ===\n");

    // 1. 基础运算符重载
    println!("1. 运算符重载:");
    let p1 = Point { x: 1.0, y: 2.0 };
    let p2 = Point { x: 3.0, y: 4.0 };
    let p3 = p1 + p2;
    println!("{} + {} = {}", p1, p2, p3);

    // 2. trait 方法调用
    println!("\n2. Trait 方法:");
    p1.draw();
    p1.describe();

    // 3. 不同类型实现同一 trait
    println!("\n3. 不同类型的 trait 实现:");
    let circle = Circle::new(Point { x: 0.0, y: 0.0 }, 5.0);
    circle.draw();
    circle.describe();

    // 4. trait 对象
    println!("\n4. Trait 对象:");
    let shapes: Vec<&dyn Drawable> = vec![&p1, &circle];
    for shape in shapes {
        draw_shape(shape);
        println!();
    }

    // 演示泛型函数（需要具体类型）
    println!("4.1 泛型函数调用:");
    draw_shape_gen(&p1);
    println!();
    draw_shape_gen(&circle);
    println!();

    // 5. 泛型 trait
    println!("5. 泛型 trait 比较:");
    let p4 = Point { x: 5.0, y: 0.0 };
    let p5 = Point { x: 3.0, y: 4.0 };

    match p4.compare(&p5) {
        std::cmp::Ordering::Less => println!("{} 距离原点更近", p4),
        std::cmp::Ordering::Greater => println!("{} 距离原点更远", p4),
        std::cmp::Ordering::Equal => println!("{} 和 {} 距离原点相等", p4, p5),
    }

    // 6. trait bound 示例
    println!("\n6. Trait Bound:");
    compare_and_draw(&p4, &p5);
}
