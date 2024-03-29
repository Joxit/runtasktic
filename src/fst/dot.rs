use crate::fst::TaskFst;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

pub fn dot_write_file<W: Write>(fst: &TaskFst, writer: &mut W) -> std::io::Result<()> {
  writeln!(writer, "digraph {{")?;
  let mut iter = fst.iter();
  while iter.has_next() {
    let node = iter.next().unwrap();
    let label = node.label.replace("\"", "\\\"");
    let id = format_id(&node.label);

    let shape = if node.next.len() == 0 {
      "doublecircle"
    } else {
      "circle"
    };

    if node.prev.len() == 0 {
      writeln!(writer, r#"  init_{}[label="", shape=point]"#, id)?;
      writeln!(writer, r#"  init_{} -> {0}"#, id)?;
    }

    writeln!(writer, r#"  {}[label="{}" shape={}]"#, id, label, shape)?;
    for next in node.next {
      writeln!(writer, "  {} -> {}", id, format_id(&fst.states[next].label))?;
    }
    iter.mark_done(node.id);
  }
  writeln!(writer, "}}")?;
  Ok(())
}

pub fn dot_write_png<R: Read, P: AsRef<Path>>(reader: &mut R, path: P) -> std::io::Result<()> {
  let file = File::create(path)?;
  let mut r: Vec<u8> = vec![];
  reader.read_to_end(&mut r)?;
  let mut child = Command::new("dot")
    .arg("-T")
    .arg("png")
    .stdin(Stdio::piped())
    .stdout(file)
    .spawn()?;

  if let Some(stdin) = &mut child.stdin {
    stdin.write_all(&r)?;
  }

  child.wait()?;

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
    fst.add_state("\"a\"");
    fst.add_state("b a ba");
    fst.add_arc(0, 1);
    fst.add_start_state(0);

    fst.add_state("c");
    fst.add_arc(0, 2);
    fst.add_arc(1, 2);

    fst.add_state("d%s");
    fst.add_arc(2, 3);

    fst.add_state("e");
    fst.add_start_state(4);

    fst.add_arc(4, 3);

    let mut result: Vec<u8> = vec![];
    assert!(super::dot_write_file(&fst, &mut result).is_ok());
    let result = std::str::from_utf8(&result).unwrap();
    println!("{}", result);
    assert_eq!(
      result,
      r#"digraph {
  init_a[label="", shape=point]
  init_a -> a
  a[label="\"a\"" shape=circle]
  a -> b_a_ba
  a -> c
  init_e[label="", shape=point]
  init_e -> e
  e[label="e" shape=circle]
  e -> ds
  b_a_ba[label="b a ba" shape=circle]
  b_a_ba -> c
  c[label="c" shape=circle]
  c -> ds
  ds[label="d%s" shape=doublecircle]
}
"#
    );
  }
}
