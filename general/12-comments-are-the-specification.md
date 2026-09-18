# 12. Comments are the specification

**Class:** documentation defect that becomes a code defect at the next edit.
**Severity:** whatever the next person builds on the comment.

## The pattern

In a codebase where the comments explain the rules, the comments **are** read as the
rules, by the next reviewer and by the next author. A comment that promises more than the
code does is therefore a defect even when the code is the safer of the two.

Four found in one project:

- "every failure falls back to the full price", above code where one failure returned an
  encoding error and refused the transaction;
- "creation is gated to a privileged key", two revisions after it became token-gated;
- an encoder whose comment said it **refuses** an oversized field, above code that clamps
  the length byte and writes every byte anyway (the TypeScript twin of the same encoder
  did refuse; the two codecs for one wire format disagreed about what is writable);
- "whoever completes the transaction keeps the deposit, which is roughly what it costs
  them": the estimate was wrong by a factor of eight million and the design built on it
  was exploitable.

## The rule

Every "roughly", "about", "should never" and "cannot" in a comment is a claim. Measure
the numeric ones once. For the others, find the line that enforces them; if there is
none, the comment is describing an intention, and the intention needs either code or a
weaker sentence.

## Two smaller habits that follow

- When the fix is to the comment and not the code, say so in the commit: the binary must
  not change, and that is worth proving (build twice from the unchanged tree, once after
  the edit, compare hashes).
- Moving a line in a shared crate changes every binary that links it, even ones that use
  nothing that moved: with `overflow-checks` on, each bounds check carries a
  `core::panic::Location` with the source line. One byte differed at `0xdc0`. A
  reproducible-build check that says MISMATCH for a comment is telling the truth, and the
  deployment record should carry the commit each binary was built from so the check can
  say "moved" instead.
