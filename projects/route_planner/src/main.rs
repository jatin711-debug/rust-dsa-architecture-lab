//! File parsing and a graph algorithm behind a small command line boundary.

use std::env;
use std::error::Error;
use std::fs;

use rust_learning_and_dsa::graph::Graph;

fn parse_graph(input: &str) -> Result<Graph<usize>, String> {
    let mut lines = input.lines().enumerate().filter(|(_, line)| {
        let trimmed = line.trim();
        !trimmed.is_empty() && !trimmed.starts_with('#')
    });
    let (_, count_line) = lines.next().ok_or("missing vertex count")?;
    let count: usize = count_line
        .trim()
        .parse()
        .map_err(|_| "invalid vertex count")?;
    let mut graph = Graph::new();
    for _ in 0..count {
        graph.add_vertex();
    }
    for (line_number, line) in lines {
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(format!(
                "line {}: expected 'from to weight'",
                line_number + 1
            ));
        }
        let numbers: Vec<usize> = parts
            .iter()
            .map(|part| part.parse::<usize>())
            .collect::<Result<_, _>>()
            .map_err(|_| format!("line {}: expected nonnegative integers", line_number + 1))?;
        if numbers[0] >= count || numbers[1] >= count {
            return Err(format!("line {}: vertex out of range", line_number + 1));
        }
        graph.add_edge(numbers[0], numbers[1], numbers[2]);
    }
    Ok(graph)
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let usage = "usage: route_planner <graph-file> <start> <goal>";
    let path = args.next().ok_or(usage)?;
    let start: usize = args.next().ok_or(usage)?.parse()?;
    let goal: usize = args.next().ok_or(usage)?.parse()?;
    if args.next().is_some() {
        return Err(usage.into());
    }
    let graph = parse_graph(&fs::read_to_string(path)?)?;
    if start >= graph.vertex_count() || goal >= graph.vertex_count() {
        return Err("start or goal vertex out of range".into());
    }
    match graph.dijkstra_path(start, goal) {
        Some((cost, path)) => println!("cost={cost} path={path:?}"),
        None => println!("unreachable"),
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_routes() {
        let graph = parse_graph("# vertices\n4\n0 1 2\n1 3 3\n0 3 20\n").unwrap();
        assert_eq!(graph.dijkstra_path(0, 3), Some((5, vec![0, 1, 3])));
    }

    #[test]
    fn invalid_edges_are_errors() {
        assert!(parse_graph("2\n0 2 1\n").is_err());
        assert!(parse_graph("2\n0 1 -1\n").is_err());
    }
}
