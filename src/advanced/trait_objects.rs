//! Dynamic dispatch: `dyn Trait` vs generics (static dispatch).
//!
//! Generics monomorphize: the compiler generates a specialized copy of the
//! function for every concrete type, so calls are direct and can be inlined —
//! but the binary grows, and you can't store *mixed* types in one collection.
//!
//! `dyn Trait` uses a **vtable**: a table of function pointers per concrete
//! type, referenced through a fat pointer `(data, vtable)`. Calls go through
//! an indirection (slower, not inlinable), but one `Box<dyn Trait>` can hold
//! any implementor, and adding a new type needs no recompilation of callers.
//!
//! ```text
//! Box<dyn Shape>  ──► data: &Circle        vtable for Circle
//!                    └──┐                  ┌── area    -> Circle::area
//!                       │                  ├── name    -> Circle::name
//!                       └──► (ptr, vtable) └── drop    -> drop_in_place
//! ```
//!
//! Rule of thumb: generics by default; `dyn` when you need type erasure
//! (heterogeneous collections, plugin-style extensibility, callbacks).

use std::fmt;

/// The trait we dispatch over.
pub trait Shape {
    /// Area of the shape in square units.
    fn area(&self) -> f64;
    /// Human-readable name.
    fn name(&self) -> &'static str;
}

/// A circle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    /// Radius.
    pub radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    fn name(&self) -> &'static str {
        "circle"
    }
}

/// A square.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Square {
    /// Side length.
    pub side: f64,
}

impl Shape for Square {
    fn area(&self) -> f64 {
        self.side * self.side
    }

    fn name(&self) -> &'static str {
        "square"
    }
}

/// A right triangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle {
    /// Base length.
    pub base: f64,
    /// Height.
    pub height: f64,
}

impl Shape for Triangle {
    fn area(&self) -> f64 {
        0.5 * self.base * self.height
    }

    fn name(&self) -> &'static str {
        "triangle"
    }
}

/// Total area of a homogeneous collection — **static dispatch**. One
/// monomorphized copy per `S`.
pub fn total_area_static<S: Shape>(shapes: &[S]) -> f64 {
    shapes.iter().map(Shape::area).sum()
}

/// Total area of a heterogeneous collection — **dynamic dispatch** through a
/// vtable. One shared implementation for all `Shape`s.
pub fn total_area_dyn(shapes: &[&dyn Shape]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

/// Total area of owned, boxed shapes. O(1) to build per shape; the `Box`
/// itself is the fat pointer.
#[must_use]
pub fn total_area_boxed(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

/// A dispatch function for `&dyn Shape`, demonstrating that `dyn` values can
/// be passed around like any other reference.
pub fn describe(shape: &dyn Shape) -> String {
    format!("a {} with area {:.2}", shape.name(), shape.area())
}

/// How many bytes a fat pointer costs on this platform (informational).
#[must_use]
pub fn fat_pointer_size() -> usize {
    std::mem::size_of::<&dyn Shape>()
}

impl fmt::Debug for dyn Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (area {:.2})", self.name(), self.area())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_shapes() -> (Vec<Circle>, Vec<Square>, Vec<Triangle>) {
        (
            vec![Circle { radius: 1.0 }, Circle { radius: 2.0 }],
            vec![Square { side: 3.0 }],
            vec![Triangle {
                base: 4.0,
                height: 5.0,
            }],
        )
    }

    #[test]
    fn static_dispatch_sums_correctly() {
        let (circles, squares, triangles) = sample_shapes();
        let circle_area = total_area_static(&circles);
        let square_area = total_area_static(&squares);
        let triangle_area = total_area_static(&triangles);
        assert!((circle_area - std::f64::consts::PI * 5.0).abs() < 1e-9);
        assert!((square_area - 9.0).abs() < 1e-9);
        assert!((triangle_area - 10.0).abs() < 1e-9);
    }

    #[test]
    fn dynamic_dispatch_mixes_types() {
        let circle = Circle { radius: 1.0 };
        let square = Square { side: 3.0 };
        let triangle = Triangle {
            base: 4.0,
            height: 5.0,
        };
        let shapes: [&dyn Shape; 3] = [&circle, &square, &triangle];
        let total = total_area_dyn(&shapes);
        assert!((total - (std::f64::consts::PI + 9.0 + 10.0)).abs() < 1e-9);
    }

    #[test]
    #[allow(clippy::float_cmp)] // exact PI sums are deterministic here
    fn boxed_shapes_own_their_data() {
        let shapes: Vec<Box<dyn Shape>> = vec![
            Box::new(Circle { radius: 2.0 }),
            Box::new(Square { side: 4.0 }),
        ];
        // The Box<dyn Shape> does not implement Shape for free; dispatch
        // through the deref when calling trait methods.
        let total = total_area_boxed(&shapes);
        assert!((total - (std::f64::consts::PI * 4.0 + 16.0)).abs() < 1e-9);
    }

    #[test]
    fn describe_works_for_all() {
        let circle = Circle { radius: 1.0 };
        let square = Square { side: 2.0 };
        assert_eq!(
            describe(&circle),
            format!("a circle with area {:.2}", std::f64::consts::PI)
        );
        assert_eq!(describe(&square), "a square with area 4.00");
    }

    #[test]
    fn dyn_values_are_fat_pointers() {
        // Two words: data pointer + vtable pointer (on 64-bit: 16 bytes).
        assert!(fat_pointer_size() >= 2 * std::mem::size_of::<usize>());
    }

    #[test]
    fn debug_impl_for_dyn_shape() {
        let circle = Circle { radius: 1.0 };
        assert_eq!(
            format!("{:?}", &circle as &dyn Shape),
            format!("circle (area {:.2})", std::f64::consts::PI)
        );
    }
}
