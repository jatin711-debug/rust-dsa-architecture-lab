//! Example: exercise every data structure in the library.
//!
//! Run with: `cargo run --example all_data_structures`

use rust_learning_and_dsa::prelude::*;
fn main() {
    println!("=== DynamicArray ===");
    let mut arr = DynamicArray::new();
    arr.extend([1, 2, 3, 4, 5]);
    arr.insert(2, 99);
    println!("  {:?} (len {})", arr, arr.len());

    println!("=== Stack (LIFO) ===");
    let mut stack = Stack::new();
    stack.extend([10, 20, 30]);
    println!("  top: {:?}", stack.peek());
    while let Some(v) = stack.pop() {
        print!(" {v}");
    }
    println!();

    println!("=== Queue (FIFO) ===");
    let mut queue = Queue::new();
    queue.extend(["a", "b", "c"]);
    println!("  front: {:?}", queue.peek());
    while let Some(v) = queue.dequeue() {
        print!(" {v}");
    }
    println!();

    println!("=== SinglyLinkedList ===");
    let mut list = SinglyLinkedList::new();
    list.push_front(1);
    list.append(2);
    println!("  {:?}", list);

    println!("=== DoublyLinkedList ===");
    let mut dlist = DoublyLinkedList::new();
    dlist.push_back(1);
    dlist.push_front(0);
    dlist.push_back(2);
    println!("  front {:?}, back {:?}", dlist.front(), dlist.back());

    println!("=== HashMap ===");
    let mut map = HashMap::new();
    for i in 0..10 {
        map.insert(format!("key{i}"), i * i);
    }
    println!("  key5 -> {:?}", map.get(&"key5".to_string()));
    println!("  len = {}", map.len());

    println!("=== BinarySearchTree ===");
    let mut bst = BinarySearchTree::new();
    for v in [50, 30, 70, 20, 40, 60, 80] {
        bst.insert(v);
    }
    println!(
        "  in-order: {:?}",
        bst.in_order().copied().collect::<Vec<_>>()
    );
    println!("  min {:?}, max {:?}", bst.min(), bst.max());

    println!("=== AvlTree (self-balancing) ===");
    let mut avl = AvlTree::new();
    for v in 0..1000 {
        avl.insert(v);
    }
    println!("  height of 1000 sorted inserts = {}", avl.height());

    println!("=== BinaryHeap (max-heap) ===");
    let mut heap = BinaryHeap::new();
    for v in [4, 1, 3, 2, 16, 9, 10] {
        heap.push(v);
    }
    let mut sorted = Vec::new();
    while let Some(v) = heap.pop() {
        sorted.push(v);
    }
    println!("  popped: {:?}", sorted);

    println!("=== Graph (BFS + Dijkstra) ===");
    let mut graph: Graph<usize> = Graph::new();
    for _ in 0..5 {
        graph.add_vertex();
    }
    graph.add_undirected_edge(0, 1, 1);
    graph.add_undirected_edge(1, 2, 1);
    graph.add_undirected_edge(2, 3, 1);
    graph.add_undirected_edge(3, 4, 1);
    println!("  bfs distances from 0: {:?}", graph.bfs_distances(0));
    println!(
        "  dijkstra distances from 0: {:?}",
        graph.dijkstra_distances(0)
    );

    println!("=== Trie (prefix tree) ===");
    let mut trie = Trie::new();
    for word in ["rust", "rustacean", "rusty", "reliable", "rapid"] {
        trie.insert(word);
    }
    println!("  autocomplete(\"ru\") = {:?}", trie.autocomplete("ru"));

    println!("=== UnionFind (disjoint sets) ===");
    let mut uf = UnionFind::new(6);
    for (a, b) in [(0, 1), (1, 2), (3, 4)] {
        uf.union(a, b);
    }
    println!(
        "  0-2 connected: {}, components: {}",
        uf.connected(0, 2),
        uf.components()
    );

    println!("=== Sorting algorithms ===");
    let data = vec![9, 3, 7, 1, 8, 2, 6, 4, 5, 0];
    let mut q = data.clone();
    quicksort(&mut q);
    println!("  quicksort:  {:?}", q);
    let mut m = data.clone();
    mergesort(&mut m);
    println!("  mergesort:  {:?}", m);
    let mut h = data.clone();
    heapsort(&mut h);
    println!("  heapsort:   {:?}", h);
}
