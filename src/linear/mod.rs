//! Linear data structures: sequences and adapters built on top of them.

pub mod doubly_linked_list;
pub mod dynamic_array;
pub mod queue;
pub mod singly_linked_list;
pub mod stack;

pub use doubly_linked_list::DoublyLinkedList;
pub use dynamic_array::DynamicArray;
pub use queue::Queue;
pub use singly_linked_list::SinglyLinkedList;
pub use stack::Stack;
