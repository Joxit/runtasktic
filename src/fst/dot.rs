use crate::fst::TaskFst;
use std::io::Write;

pub fn dot_write_file<W: Write>(fst: &TaskFst, writer: &mut W) -> std::io::Result<()> {
  writeln!(writer, "digraph {{")?;
  let mut iter = fst.iter();
  while iter.has_next() {
    let node = iter.next().unwrap();
    let label = node.label.replace("\"", "\\\"");
    let id = format_id(&node.label);
    let shape = if node.prev.len() == 0 {
      "doublecircle"
    } else {
      "circle"
    };
    writeln!(writer, r#"  {}[label="{}" shape={}]"#, id, label, shape)?;
    for next in node.next {
      writeln!(writer, "  {} -> {}", id, format_id(&fst.states[next].label))?;
    }
    iter.mark_done(node.id);
  }
  writeln!(writer, "}}")?;
  Ok(())
}

fn format_id(id: &String) -> String {
  id.chars()
    .map(|c| if c == ' ' { '_' } else { c })
    .filter(|c| c.is_alphanumeric() || *c == '_')
    .collect::<String>()
}

#[cfg(test)]
mod test {
  use super::*;
  #[test]
  fn dot_write_file() {
    let mut fst = TaskFst::new();
    fst.add_state("\"a\"".to_string());
    fst.add_state("b a ba".to_string());
    fst.add_arc(0, 1);
    fst.add_start_state(0);

    fst.add_state("c".to_string());
    fst.add_arc(0, 2);
    fst.add_arc(1, 2);

    fst.add_state("d%s".to_string());
    fst.add_arc(2, 3);

    fst.add_state("e".to_string());
    fst.add_start_state(4);

    fst.add_arc(4, 3);

    let mut result: Vec<u8> = vec![];
    assert!(super::dot_write_file(&fst, &mut result).is_ok());
    let result = std::str::from_utf8(&result).unwrap();
    println!("{}", result);
    assert_eq!(result, r#"digraph {
  a[label="\"a\"" shape=doublecircle]
  a -> b_a_ba
  a -> c
  e[label="e" shape=doublecircle]
  e -> ds
  b_a_ba[label="b a ba" shape=circle]
  b_a_ba -> c
  c[label="c" shape=circle]
  c -> ds
  ds[label="d%s" shape=circle]
}
"#);
  }
}
