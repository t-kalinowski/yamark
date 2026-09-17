# Exact reviewed/current differences

Historical expected output was reproduced with PR #6 head `2e2caa4b14b8af0464a836988ec1967336762b78`. Current output is from `0b1f072e710daa58ce4da412d8efc68f7d0fcd20`. All invocations below returned status 0 and empty stderr on both revisions.

Each command in an entry uses the same input and expectations. Escaped strings use JSON notation; `\r`, `\n`, `\t`, `\u000b`, and `\u000c` retain the exact UTF-8 bytes. These are evidence, not executable expectations.

## pr6-027

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-027) · [active transcript](../../cases/recovered_pr6_preserves_literals_and_normalizes_real_links_027.case) · [reviewed transcript](recovered_pr6_preserves_literals_and_normalizes_real_links_027.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 Before
-$a \$ _literal_ [x](  url  )$ after [real](target).
+$a \$ *literal* [x](  url  )$ after [real](target).
```

```json
{
  "stdin": "Before\n$a \\$ _literal_ [x](  url  )$ after [real](  target  ).\n",
  "reviewed_stdout": "Before\n$a \\$ _literal_ [x](  url  )$ after [real](target).\n",
  "current_stdout": "Before\n$a \\$ *literal* [x](  url  )$ after [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-028

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-028)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 Before
-$a \$ _literal_ [x](  url  )$ after [real](target).
+$a \$ *literal* [x](  url  )$ after [real](target).
```

```json
{
  "stdin": "Before\n$a \\$ _literal_ [x](  url  )$ after [real](target).\n",
  "reviewed_stdout": "Before\n$a \\$ _literal_ [x](  url  )$ after [real](target).\n",
  "current_stdout": "Before\n$a \\$ *literal* [x](  url  )$ after [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-031

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-031) · [active transcript](../../cases/recovered_pr6_preserves_literals_and_normalizes_real_links_031.case) · [reviewed transcript](recovered_pr6_preserves_literals_and_normalizes_real_links_031.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-Before $a \$ _literal_ [x](  url  )$ after [real](target).
+Before $a \$ *literal* [x](  url  )$ after [real](target).
```

```json
{
  "stdin": "Before\n$a \\$ _literal_ [x](  url  )$ after [real](  target  ).\n",
  "reviewed_stdout": "Before $a \\$ _literal_ [x](  url  )$ after [real](target).\n",
  "current_stdout": "Before $a \\$ *literal* [x](  url  )$ after [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-032

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-032)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-Before $a \$ _literal_ [x](  url  )$ after [real](target).
+Before $a \$ *literal* [x](  url  )$ after [real](target).
```

```json
{
  "stdin": "Before $a \\$ _literal_ [x](  url  )$ after [real](target).\n",
  "reviewed_stdout": "Before $a \\$ _literal_ [x](  url  )$ after [real](target).\n",
  "current_stdout": "Before $a \\$ *literal* [x](  url  )$ after [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-035

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-035) · [active transcript](../../cases/recovered_pr6_preserves_literals_and_normalizes_real_links_035.case) · [reviewed transcript](recovered_pr6_preserves_literals_and_normalizes_real_links_035.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-- Before $a \$ _literal_ [x](  url  )$ after [real](target).
+- Before $a \$ *literal* [x](  url  )$ after [real](target).
```

```json
{
  "stdin": "- Before\n  $a \\$ _literal_ [x](  url  )$ after [real](  target  ).\n",
  "reviewed_stdout": "- Before $a \\$ _literal_ [x](  url  )$ after [real](target).\n",
  "current_stdout": "- Before $a \\$ *literal* [x](  url  )$ after [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-036

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-036)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-- Before $a \$ _literal_ [x](  url  )$ after [real](target).
+- Before $a \$ *literal* [x](  url  )$ after [real](target).
```

```json
{
  "stdin": "- Before $a \\$ _literal_ [x](  url  )$ after [real](target).\n",
  "reviewed_stdout": "- Before $a \\$ _literal_ [x](  url  )$ after [real](target).\n",
  "current_stdout": "- Before $a \\$ *literal* [x](  url  )$ after [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-039

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-039) · [active transcript](../../cases/recovered_pr6_preserves_literals_and_normalizes_real_links_039.case) · [reviewed transcript](recovered_pr6_preserves_literals_and_normalizes_real_links_039.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-> Before $a \$ _literal_ [x](  url  )$ after [real](target).
+> Before $a \$ *literal* [x](  url  )$ after [real](target).
```

```json
{
  "stdin": "> Before\n> $a \\$ _literal_ [x](  url  )$ after [real](  target  ).\n",
  "reviewed_stdout": "> Before $a \\$ _literal_ [x](  url  )$ after [real](target).\n",
  "current_stdout": "> Before $a \\$ *literal* [x](  url  )$ after [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-040

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-040)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-> Before $a \$ _literal_ [x](  url  )$ after [real](target).
+> Before $a \$ *literal* [x](  url  )$ after [real](target).
```

```json
{
  "stdin": "> Before $a \\$ _literal_ [x](  url  )$ after [real](target).\n",
  "reviewed_stdout": "> Before $a \\$ _literal_ [x](  url  )$ after [real](target).\n",
  "current_stdout": "> Before $a \\$ *literal* [x](  url  )$ after [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-089

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-089) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_089.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_089.case)

```text
format --stdin-file-path input.md --wrap none
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Before `first  
+Before `first \
   second [x]( url )` after _outside_.
```

```json
{
  "stdin": "Before `first  \n  second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "Before `first  \n  second [x]( url )` after _outside_.\n",
  "current_stdout": "Before `first \\\n  second [x]( url )` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-090

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-090) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_090.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_090.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Before `first  
+Before `first \
   second [x]( url )` after *outside*.
```

```json
{
  "stdin": "Before `first  \n  second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "Before `first  \n  second [x]( url )` after *outside*.\n",
  "current_stdout": "Before `first \\\n  second [x]( url )` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-091

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-091)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Before `first  
+Before `first \
   second [x]( url )` after *outside*.
```

```json
{
  "stdin": "Before `first  \n  second [x]( url )` after *outside*.\n",
  "reviewed_stdout": "Before `first  \n  second [x]( url )` after *outside*.\n",
  "current_stdout": "Before `first \\\n  second [x]( url )` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-092

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-092) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_092.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_092.case)

```text
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 80
format --stdin-file-path input.md --wrap sentence:80
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Before `first  
+Before `first
   second [x]( url )` after _outside_.
```

```json
{
  "stdin": "Before `first  \n  second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "Before `first  \n  second [x]( url )` after _outside_.\n",
  "current_stdout": "Before `first\n  second [x]( url )` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-093

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-093) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_093.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_093.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Before `first  
-  second [x]( url )` after *outside*.
+Before `first
+  second [x]( url )` after _outside_.
```

```json
{
  "stdin": "Before `first  \n  second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "Before `first  \n  second [x]( url )` after *outside*.\n",
  "current_stdout": "Before `first\n  second [x]( url )` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-094

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-094)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Before `first  
+Before `first
   second [x]( url )` after *outside*.
```

```json
{
  "stdin": "Before `first  \n  second [x]( url )` after *outside*.\n",
  "reviewed_stdout": "Before `first  \n  second [x]( url )` after *outside*.\n",
  "current_stdout": "Before `first\n  second [x]( url )` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-095

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-095) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_095.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_095.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 80
format --stdin-file-path input.md --wrap sentence:80
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Before `first  
+- Before `first
     second [x]( url )` after _outside_.
```

```json
{
  "stdin": "- Before `first  \n    second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "- Before `first  \n    second [x]( url )` after _outside_.\n",
  "current_stdout": "- Before `first\n    second [x]( url )` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-096

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-096) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_096.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_096.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Before `first  
-    second [x]( url )` after *outside*.
+- Before `first
+    second [x]( url )` after _outside_.
```

```json
{
  "stdin": "- Before `first  \n    second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "- Before `first  \n    second [x]( url )` after *outside*.\n",
  "current_stdout": "- Before `first\n    second [x]( url )` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-097

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-097)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Before `first  
+- Before `first
     second [x]( url )` after *outside*.
```

```json
{
  "stdin": "- Before `first  \n    second [x]( url )` after *outside*.\n",
  "reviewed_stdout": "- Before `first  \n    second [x]( url )` after *outside*.\n",
  "current_stdout": "- Before `first\n    second [x]( url )` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-098

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-098) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_098.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_098.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 80
format --stdin-file-path input.md --wrap sentence:80
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> Before `first  
+> Before `first
 >   second [x]( url )` after _outside_.
```

```json
{
  "stdin": "> Before `first  \n>   second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "> Before `first  \n>   second [x]( url )` after _outside_.\n",
  "current_stdout": "> Before `first\n>   second [x]( url )` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-099

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-099) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_099.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_099.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> Before `first  
->   second [x]( url )` after *outside*.
+> Before `first
+>   second [x]( url )` after _outside_.
```

```json
{
  "stdin": "> Before `first  \n>   second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "> Before `first  \n>   second [x]( url )` after *outside*.\n",
  "current_stdout": "> Before `first\n>   second [x]( url )` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-100

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-100)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> Before `first  
+> Before `first
 >   second [x]( url )` after *outside*.
```

```json
{
  "stdin": "> Before `first  \n>   second [x]( url )` after *outside*.\n",
  "reviewed_stdout": "> Before `first  \n>   second [x]( url )` after *outside*.\n",
  "current_stdout": "> Before `first\n>   second [x]( url )` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-101

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-101) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_101.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_101.case)

```text
format --stdin-file-path input.md --wrap none
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> > Before `first  
+> > Before `first
 > >   second [x]( url )` after _outside_.
```

```json
{
  "stdin": "> > Before `first  \n> >   second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "> > Before `first  \n> >   second [x]( url )` after _outside_.\n",
  "current_stdout": "> > Before `first\n> >   second [x]( url )` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-102

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-102) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_102.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_102.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> > Before `first  
-> >   second [x]( url )` after *outside*.
+> > Before `first
+> >   second [x]( url )` after _outside_.
```

```json
{
  "stdin": "> > Before `first  \n> >   second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "> > Before `first  \n> >   second [x]( url )` after *outside*.\n",
  "current_stdout": "> > Before `first\n> >   second [x]( url )` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-103

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-103)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> > Before `first  
+> > Before `first
 > >   second [x]( url )` after *outside*.
```

```json
{
  "stdin": "> > Before `first  \n> >   second [x]( url )` after *outside*.\n",
  "reviewed_stdout": "> > Before `first  \n> >   second [x]( url )` after *outside*.\n",
  "current_stdout": "> > Before `first\n> >   second [x]( url )` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-104

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-104) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_104.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_104.case)

```text
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 80
format --stdin-file-path input.md --wrap sentence:80
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> > Before `first  
-> >   second [x]( url )` after _outside_.
+> > Before `first second [x]( url )` after _outside_.
```

```json
{
  "stdin": "> > Before `first  \n> >   second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "> > Before `first  \n> >   second [x]( url )` after _outside_.\n",
  "current_stdout": "> > Before `first second [x]( url )` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-105

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-105) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_105.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_105.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> > Before `first  
-> >   second [x]( url )` after *outside*.
+> > Before `first second [x]( url )` after *outside*.
```

```json
{
  "stdin": "> > Before `first  \n> >   second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "> > Before `first  \n> >   second [x]( url )` after *outside*.\n",
  "current_stdout": "> > Before `first second [x]( url )` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-106

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-106)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> > Before `first  
-> >   second [x]( url )` after *outside*.
+> > Before `first second [x]( url )` after *outside*.
```

```json
{
  "stdin": "> > Before `first  \n> >   second [x]( url )` after *outside*.\n",
  "reviewed_stdout": "> > Before `first  \n> >   second [x]( url )` after *outside*.\n",
  "current_stdout": "> > Before `first second [x]( url )` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-110

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-110) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_110.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_110.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 Before `first\
-  second [x]( url )` after *outside*.
+  second [x]( url )` after _outside_.
```

```json
{
  "stdin": "Before `first\\\n  second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "Before `first\\\n  second [x]( url )` after *outside*.\n",
  "current_stdout": "Before `first\\\n  second [x]( url )` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-112

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-112) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_112.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_112.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 - Before `first\
-    second [x]( url )` after *outside*.
+    second [x]( url )` after _outside_.
```

```json
{
  "stdin": "- Before `first\\\n    second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "- Before `first\\\n    second [x]( url )` after *outside*.\n",
  "current_stdout": "- Before `first\\\n    second [x]( url )` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-115

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-115) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_115.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_115.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 > Before `first\
->   second [x]( url )` after *outside*.
+>   second [x]( url )` after _outside_.
```

```json
{
  "stdin": "> Before `first\\\n>   second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "> Before `first\\\n>   second [x]( url )` after *outside*.\n",
  "current_stdout": "> Before `first\\\n>   second [x]( url )` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-118

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-118) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_118.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_118.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 > > Before `first\
-> >   second [x]( url )` after *outside*.
+> >   second [x]( url )` after _outside_.
```

```json
{
  "stdin": "> > Before `first\\\n> >   second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "> > Before `first\\\n> >   second [x]( url )` after *outside*.\n",
  "current_stdout": "> > Before `first\\\n> >   second [x]( url )` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-120

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-120) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_120.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_120.case)

```text
format --stdin-file-path input.md --wrap none
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Before $first  
+Before $first \
   second \$ [x]( url )$ after _outside_.
```

```json
{
  "stdin": "Before $first  \n  second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stdout": "Before $first  \n  second \\$ [x]( url )$ after _outside_.\n",
  "current_stdout": "Before $first \\\n  second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-121

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-121) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_121.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_121.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Before $first  
+Before $first \
   second \$ [x]( url )$ after *outside*.
```

```json
{
  "stdin": "Before $first  \n  second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stdout": "Before $first  \n  second \\$ [x]( url )$ after *outside*.\n",
  "current_stdout": "Before $first \\\n  second \\$ [x]( url )$ after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-122

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-122)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Before $first  
+Before $first \
   second \$ [x]( url )$ after *outside*.
```

```json
{
  "stdin": "Before $first  \n  second \\$ [x]( url )$ after *outside*.\n",
  "reviewed_stdout": "Before $first  \n  second \\$ [x]( url )$ after *outside*.\n",
  "current_stdout": "Before $first \\\n  second \\$ [x]( url )$ after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-123

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-123) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_123.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_123.case)

```text
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 80
format --stdin-file-path input.md --wrap sentence:80
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Before $first  
+Before $first
   second \$ [x]( url )$ after _outside_.
```

```json
{
  "stdin": "Before $first  \n  second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stdout": "Before $first  \n  second \\$ [x]( url )$ after _outside_.\n",
  "current_stdout": "Before $first\n  second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-124

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-124) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_124.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_124.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Before $first  
-  second \$ [x]( url )$ after *outside*.
+Before $first
+  second \$ [x]( url )$ after _outside_.
```

```json
{
  "stdin": "Before $first  \n  second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stdout": "Before $first  \n  second \\$ [x]( url )$ after *outside*.\n",
  "current_stdout": "Before $first\n  second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-125

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-125)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Before $first  
+Before $first
   second \$ [x]( url )$ after *outside*.
```

```json
{
  "stdin": "Before $first  \n  second \\$ [x]( url )$ after *outside*.\n",
  "reviewed_stdout": "Before $first  \n  second \\$ [x]( url )$ after *outside*.\n",
  "current_stdout": "Before $first\n  second \\$ [x]( url )$ after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-126

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-126) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_126.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_126.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 80
format --stdin-file-path input.md --wrap sentence:80
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Before $first  
+- Before $first
     second \$ [x]( url )$ after _outside_.
```

```json
{
  "stdin": "- Before $first  \n    second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stdout": "- Before $first  \n    second \\$ [x]( url )$ after _outside_.\n",
  "current_stdout": "- Before $first\n    second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-127

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-127) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_127.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_127.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Before $first  
-    second \$ [x]( url )$ after *outside*.
+- Before $first
+    second \$ [x]( url )$ after _outside_.
```

```json
{
  "stdin": "- Before $first  \n    second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stdout": "- Before $first  \n    second \\$ [x]( url )$ after *outside*.\n",
  "current_stdout": "- Before $first\n    second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-128

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-128)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Before $first  
+- Before $first
     second \$ [x]( url )$ after *outside*.
```

```json
{
  "stdin": "- Before $first  \n    second \\$ [x]( url )$ after *outside*.\n",
  "reviewed_stdout": "- Before $first  \n    second \\$ [x]( url )$ after *outside*.\n",
  "current_stdout": "- Before $first\n    second \\$ [x]( url )$ after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-129

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-129) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_129.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_129.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 80
format --stdin-file-path input.md --wrap sentence:80
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> Before $first  
+> Before $first
 >   second \$ [x]( url )$ after _outside_.
```

```json
{
  "stdin": "> Before $first  \n>   second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stdout": "> Before $first  \n>   second \\$ [x]( url )$ after _outside_.\n",
  "current_stdout": "> Before $first\n>   second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-130

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-130) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_130.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_130.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> Before $first  
->   second \$ [x]( url )$ after *outside*.
+> Before $first
+>   second \$ [x]( url )$ after _outside_.
```

```json
{
  "stdin": "> Before $first  \n>   second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stdout": "> Before $first  \n>   second \\$ [x]( url )$ after *outside*.\n",
  "current_stdout": "> Before $first\n>   second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-131

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-131)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> Before $first  
+> Before $first
 >   second \$ [x]( url )$ after *outside*.
```

```json
{
  "stdin": "> Before $first  \n>   second \\$ [x]( url )$ after *outside*.\n",
  "reviewed_stdout": "> Before $first  \n>   second \\$ [x]( url )$ after *outside*.\n",
  "current_stdout": "> Before $first\n>   second \\$ [x]( url )$ after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-132

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-132) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_132.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_132.case)

```text
format --stdin-file-path input.md --wrap none
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> > Before $first  
+> > Before $first
 > >   second \$ [x]( url )$ after _outside_.
```

```json
{
  "stdin": "> > Before $first  \n> >   second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stdout": "> > Before $first  \n> >   second \\$ [x]( url )$ after _outside_.\n",
  "current_stdout": "> > Before $first\n> >   second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-133

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-133) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_133.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_133.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> > Before $first  
-> >   second \$ [x]( url )$ after *outside*.
+> > Before $first
+> >   second \$ [x]( url )$ after _outside_.
```

```json
{
  "stdin": "> > Before $first  \n> >   second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stdout": "> > Before $first  \n> >   second \\$ [x]( url )$ after *outside*.\n",
  "current_stdout": "> > Before $first\n> >   second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-134

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-134)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> > Before $first  
+> > Before $first
 > >   second \$ [x]( url )$ after *outside*.
```

```json
{
  "stdin": "> > Before $first  \n> >   second \\$ [x]( url )$ after *outside*.\n",
  "reviewed_stdout": "> > Before $first  \n> >   second \\$ [x]( url )$ after *outside*.\n",
  "current_stdout": "> > Before $first\n> >   second \\$ [x]( url )$ after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-135

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-135) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_135.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_135.case)

```text
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 80
format --stdin-file-path input.md --wrap sentence:80
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> > Before $first  
-> >   second \$ [x]( url )$ after _outside_.
+> > Before $first second \$ [x]( url )$ after _outside_.
```

```json
{
  "stdin": "> > Before $first  \n> >   second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stdout": "> > Before $first  \n> >   second \\$ [x]( url )$ after _outside_.\n",
  "current_stdout": "> > Before $first second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-136

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-136) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_136.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_136.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> > Before $first  
-> >   second \$ [x]( url )$ after *outside*.
+> > Before $first second \$ [x]( url )$ after *outside*.
```

```json
{
  "stdin": "> > Before $first  \n> >   second \\$ [x]( url )$ after _outside_.\n",
  "reviewed_stdout": "> > Before $first  \n> >   second \\$ [x]( url )$ after *outside*.\n",
  "current_stdout": "> > Before $first second \\$ [x]( url )$ after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-137

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-137)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> > Before $first  
-> >   second \$ [x]( url )$ after *outside*.
+> > Before $first second \$ [x]( url )$ after *outside*.
```

```json
{
  "stdin": "> > Before $first  \n> >   second \\$ [x]( url )$ after *outside*.\n",
  "reviewed_stdout": "> > Before $first  \n> >   second \\$ [x]( url )$ after *outside*.\n",
  "current_stdout": "> > Before $first second \\$ [x]( url )$ after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-138

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-138) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_138.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_138.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 80
format --stdin-file-path input.md --wrap sentence:80
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Before `first	
+Before `first
   second {{< include file >}}` after _outside_.
```

```json
{
  "stdin": "Before `first\t\n  second {{< include file >}}` after _outside_.\n",
  "reviewed_stdout": "Before `first\t\n  second {{< include file >}}` after _outside_.\n",
  "current_stdout": "Before `first\n  second {{< include file >}}` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-139

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-139) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_139.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_139.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Before `first	
+Before `first
   second {{< include file >}}` after *outside*.
```

```json
{
  "stdin": "Before `first\t\n  second {{< include file >}}` after _outside_.\n",
  "reviewed_stdout": "Before `first\t\n  second {{< include file >}}` after *outside*.\n",
  "current_stdout": "Before `first\n  second {{< include file >}}` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-140

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-140)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Before `first	
+Before `first
   second {{< include file >}}` after *outside*.
```

```json
{
  "stdin": "Before `first\t\n  second {{< include file >}}` after *outside*.\n",
  "reviewed_stdout": "Before `first\t\n  second {{< include file >}}` after *outside*.\n",
  "current_stdout": "Before `first\n  second {{< include file >}}` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-141

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-141) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_141.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_141.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 80
format --stdin-file-path input.md --wrap sentence:80
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Before `first	
+- Before `first
     second {{< include file >}}` after _outside_.
```

```json
{
  "stdin": "- Before `first\t\n    second {{< include file >}}` after _outside_.\n",
  "reviewed_stdout": "- Before `first\t\n    second {{< include file >}}` after _outside_.\n",
  "current_stdout": "- Before `first\n    second {{< include file >}}` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-142

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-142) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_142.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_142.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Before `first	
-    second {{< include file >}}` after *outside*.
+- Before `first
+    second {{< include file >}}` after _outside_.
```

```json
{
  "stdin": "- Before `first\t\n    second {{< include file >}}` after _outside_.\n",
  "reviewed_stdout": "- Before `first\t\n    second {{< include file >}}` after *outside*.\n",
  "current_stdout": "- Before `first\n    second {{< include file >}}` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-143

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-143)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Before `first	
+- Before `first
     second {{< include file >}}` after *outside*.
```

```json
{
  "stdin": "- Before `first\t\n    second {{< include file >}}` after *outside*.\n",
  "reviewed_stdout": "- Before `first\t\n    second {{< include file >}}` after *outside*.\n",
  "current_stdout": "- Before `first\n    second {{< include file >}}` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-144

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-144) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_144.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_144.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 80
format --stdin-file-path input.md --wrap sentence:80
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> Before `first	
->   second {{< include file >}}` after _outside_.
+> Before `first second {{< include file >}}` after _outside_.
```

```json
{
  "stdin": "> Before `first\t\n>   second {{< include file >}}` after _outside_.\n",
  "reviewed_stdout": "> Before `first\t\n>   second {{< include file >}}` after _outside_.\n",
  "current_stdout": "> Before `first second {{< include file >}}` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-145

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-145) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_145.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_145.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> Before `first	
->   second {{< include file >}}` after *outside*.
+> Before `first second {{< include file >}}` after *outside*.
```

```json
{
  "stdin": "> Before `first\t\n>   second {{< include file >}}` after _outside_.\n",
  "reviewed_stdout": "> Before `first\t\n>   second {{< include file >}}` after *outside*.\n",
  "current_stdout": "> Before `first second {{< include file >}}` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-146

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-146)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> Before `first	
->   second {{< include file >}}` after *outside*.
+> Before `first second {{< include file >}}` after *outside*.
```

```json
{
  "stdin": "> Before `first\t\n>   second {{< include file >}}` after *outside*.\n",
  "reviewed_stdout": "> Before `first\t\n>   second {{< include file >}}` after *outside*.\n",
  "current_stdout": "> Before `first second {{< include file >}}` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-147

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-147) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_147.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_147.case)

```text
format --stdin-file-path input.md --wrap none
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> > Before `first	
+> > Before `first
 > >   second {{< include file >}}` after _outside_.
```

```json
{
  "stdin": "> > Before `first\t\n> >   second {{< include file >}}` after _outside_.\n",
  "reviewed_stdout": "> > Before `first\t\n> >   second {{< include file >}}` after _outside_.\n",
  "current_stdout": "> > Before `first\n> >   second {{< include file >}}` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-148

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-148) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_148.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_148.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> > Before `first	
-> >   second {{< include file >}}` after *outside*.
+> > Before `first
+> >   second {{< include file >}}` after _outside_.
```

```json
{
  "stdin": "> > Before `first\t\n> >   second {{< include file >}}` after _outside_.\n",
  "reviewed_stdout": "> > Before `first\t\n> >   second {{< include file >}}` after *outside*.\n",
  "current_stdout": "> > Before `first\n> >   second {{< include file >}}` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-149

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-149)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> > Before `first	
+> > Before `first
 > >   second {{< include file >}}` after *outside*.
```

```json
{
  "stdin": "> > Before `first\t\n> >   second {{< include file >}}` after *outside*.\n",
  "reviewed_stdout": "> > Before `first\t\n> >   second {{< include file >}}` after *outside*.\n",
  "current_stdout": "> > Before `first\n> >   second {{< include file >}}` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-150

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-150) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_150.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_150.case)

```text
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 80
format --stdin-file-path input.md --wrap sentence:80
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> > Before `first	
-> >   second {{< include file >}}` after _outside_.
+> > Before `first second {{< include file >}}` after _outside_.
```

```json
{
  "stdin": "> > Before `first\t\n> >   second {{< include file >}}` after _outside_.\n",
  "reviewed_stdout": "> > Before `first\t\n> >   second {{< include file >}}` after _outside_.\n",
  "current_stdout": "> > Before `first second {{< include file >}}` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-151

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-151) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literal_bytes_in_containers_151.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literal_bytes_in_containers_151.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> > Before `first	
-> >   second {{< include file >}}` after *outside*.
+> > Before `first second {{< include file >}}` after *outside*.
```

```json
{
  "stdin": "> > Before `first\t\n> >   second {{< include file >}}` after _outside_.\n",
  "reviewed_stdout": "> > Before `first\t\n> >   second {{< include file >}}` after *outside*.\n",
  "current_stdout": "> > Before `first second {{< include file >}}` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-152

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-152)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> > Before `first	
-> >   second {{< include file >}}` after *outside*.
+> > Before `first second {{< include file >}}` after *outside*.
```

```json
{
  "stdin": "> > Before `first\t\n> >   second {{< include file >}}` after *outside*.\n",
  "reviewed_stdout": "> > Before `first\t\n> >   second {{< include file >}}` after *outside*.\n",
  "current_stdout": "> > Before `first second {{< include file >}}` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-154

Merged #9/#10 now format eligible code or simple bare templates that #6 kept unchanged.

[Mapping](mapping.md#pr6-154) · [active transcript](../../cases/recovered_pr6_keeps_main_template_policy_154.case) · [reviewed transcript](recovered_pr6_keeps_main_template_policy_154.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 Before
-`[x]( url ) {{ value }}` after _outside_.
+`[x]( url ) {{ value }}` after *outside*.
```

```json
{
  "stdin": "Before\n`[x]( url ) {{ value }}` after _outside_.\n",
  "reviewed_stdout": "Before\n`[x]( url ) {{ value }}` after _outside_.\n",
  "current_stdout": "Before\n`[x]( url ) {{ value }}` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-155

Merged #9/#10 now format eligible code or simple bare templates that #6 kept unchanged.

[Mapping](mapping.md#pr6-155) · [active transcript](../../cases/recovered_pr6_keeps_main_template_policy_155.case) · [reviewed transcript](recovered_pr6_keeps_main_template_policy_155.case)

```text
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap paragraph
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-Before
-`[x]( url ) {{ value }}` after _outside_.
+Before `[x]( url ) {{ value }}` after _outside_.
```

```json
{
  "stdin": "Before\n`[x]( url ) {{ value }}` after _outside_.\n",
  "reviewed_stdout": "Before\n`[x]( url ) {{ value }}` after _outside_.\n",
  "current_stdout": "Before `[x]( url ) {{ value }}` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-156

Merged #9/#10 now format eligible code or simple bare templates that #6 kept unchanged.

[Mapping](mapping.md#pr6-156) · [active transcript](../../cases/recovered_pr6_keeps_main_template_policy_156.case) · [reviewed transcript](recovered_pr6_keeps_main_template_policy_156.case)

```text
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-Before
-`[x]( url ) {{ value }}` after _outside_.
+Before `[x]( url ) {{ value }}` after *outside*.
```

```json
{
  "stdin": "Before\n`[x]( url ) {{ value }}` after _outside_.\n",
  "reviewed_stdout": "Before\n`[x]( url ) {{ value }}` after _outside_.\n",
  "current_stdout": "Before `[x]( url ) {{ value }}` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-157

Merged #9/#10 now format eligible code or simple bare templates that #6 kept unchanged.

[Mapping](mapping.md#pr6-157) · [active transcript](../../cases/recovered_pr6_keeps_main_template_policy_157.case) · [reviewed transcript](recovered_pr6_keeps_main_template_policy_157.case)

```text
format --stdin-file-path input.md --wrap 20
format --stdin-file-path input.md --wrap sentence:20
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
 Before
-`[x]( url ) {{ value }}` after _outside_.
+`[x]( url ) {{ value }}`
+after _outside_.
```

```json
{
  "stdin": "Before\n`[x]( url ) {{ value }}` after _outside_.\n",
  "reviewed_stdout": "Before\n`[x]( url ) {{ value }}` after _outside_.\n",
  "current_stdout": "Before\n`[x]( url ) {{ value }}`\nafter _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-158

Merged #9/#10 now format eligible code or simple bare templates that #6 kept unchanged.

[Mapping](mapping.md#pr6-158) · [active transcript](../../cases/recovered_pr6_keeps_main_template_policy_158.case) · [reviewed transcript](recovered_pr6_keeps_main_template_policy_158.case)

```text
format --stdin-file-path input.md --wrap 20 --canonical
format --stdin-file-path input.md --wrap sentence:20 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
 Before
-`[x]( url ) {{ value }}` after _outside_.
+`[x]( url ) {{ value }}`
+after *outside*.
```

```json
{
  "stdin": "Before\n`[x]( url ) {{ value }}` after _outside_.\n",
  "reviewed_stdout": "Before\n`[x]( url ) {{ value }}` after _outside_.\n",
  "current_stdout": "Before\n`[x]( url ) {{ value }}`\nafter *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-159

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-159) · [active transcript](../../cases/recovered_pr6_keeps_main_template_policy_159.case) · [reviewed transcript](recovered_pr6_keeps_main_template_policy_159.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap 20
format --stdin-file-path input.md --wrap 20 --canonical
format --stdin-file-path input.md --wrap sentence:20
format --stdin-file-path input.md --wrap sentence:20 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Before `first  
+Before `first
   {{ value }} second` after _outside_.
```

```json
{
  "stdin": "Before `first  \n  {{ value }} second` after _outside_.\n",
  "reviewed_stdout": "Before `first  \n  {{ value }} second` after _outside_.\n",
  "current_stdout": "Before `first\n  {{ value }} second` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-162

Merged #9/#10 now format eligible code or simple bare templates that #6 kept unchanged.

[Mapping](mapping.md#pr6-162) · [active transcript](../../cases/recovered_pr6_keeps_main_template_policy_162.case) · [reviewed transcript](recovered_pr6_keeps_main_template_policy_162.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 Before {{ value }}
-more prose [real]( url ).
+more prose [real](url).
```

```json
{
  "stdin": "Before {{ value }}\nmore prose [real]( url ).\n",
  "reviewed_stdout": "Before {{ value }}\nmore prose [real]( url ).\n",
  "current_stdout": "Before {{ value }}\nmore prose [real](url).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-163

Merged #9/#10 now format eligible code or simple bare templates that #6 kept unchanged.

[Mapping](mapping.md#pr6-163) · [active transcript](../../cases/recovered_pr6_keeps_main_template_policy_163.case) · [reviewed transcript](recovered_pr6_keeps_main_template_policy_163.case)

```text
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap paragraph --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-Before {{ value }}
-more prose [real]( url ).
+Before {{ value }} more prose [real](url).
```

```json
{
  "stdin": "Before {{ value }}\nmore prose [real]( url ).\n",
  "reviewed_stdout": "Before {{ value }}\nmore prose [real]( url ).\n",
  "current_stdout": "Before {{ value }} more prose [real](url).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-164

Merged #9/#10 now format eligible code or simple bare templates that #6 kept unchanged.

[Mapping](mapping.md#pr6-164) · [active transcript](../../cases/recovered_pr6_keeps_main_template_policy_164.case) · [reviewed transcript](recovered_pr6_keeps_main_template_policy_164.case)

```text
format --stdin-file-path input.md --wrap 20
format --stdin-file-path input.md --wrap 20 --canonical
format --stdin-file-path input.md --wrap sentence:20
format --stdin-file-path input.md --wrap sentence:20 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
 Before {{ value }}
-more prose [real]( url ).
+more prose
+[real](url).
```

```json
{
  "stdin": "Before {{ value }}\nmore prose [real]( url ).\n",
  "reviewed_stdout": "Before {{ value }}\nmore prose [real]( url ).\n",
  "current_stdout": "Before {{ value }}\nmore prose\n[real](url).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-167

The raw-angle stop in merged #8 intentionally retains link padding inside HTML.

[Mapping](mapping.md#pr6-167) · [active transcript](../../cases/recovered_pr6_keeps_existing_markup_and_heading_normalization_167.case) · [reviewed transcript](recovered_pr6_keeps_existing_markup_and_heading_normalization_167.case)

```text
format --stdin-file-path input.md --wrap sentence --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-Before <kbd>[x](url)</kbd> after.
+Before <kbd>[x](  url  )</kbd> after.
```

```json
{
  "stdin": "Before <kbd>[x](  url  )</kbd> after.\n",
  "reviewed_stdout": "Before <kbd>[x](url)</kbd> after.\n",
  "current_stdout": "Before <kbd>[x](  url  )</kbd> after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-171

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-171) · [active transcript](../../cases/recovered_pr6_preserves_literal_line_endings_171.case) · [reviewed transcript](recovered_pr6_preserves_literal_line_endings_171.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-Before `first  	
+Before `first
   second\
 third` after.
```

```json
{
  "stdin": "Before `first  \t\n  second\\\nthird` after.\n",
  "reviewed_stdout": "Before `first  \t\n  second\\\nthird` after.\n",
  "current_stdout": "Before `first\n  second\\\nthird` after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-172

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-172) · [active transcript](../../cases/recovered_pr6_preserves_literal_line_endings_172.case) · [reviewed transcript](recovered_pr6_preserves_literal_line_endings_172.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-- Before `first  	
+- Before `first
     second\
   third` after.
```

```json
{
  "stdin": "- Before `first  \t\n    second\\\n  third` after.\n",
  "reviewed_stdout": "- Before `first  \t\n    second\\\n  third` after.\n",
  "current_stdout": "- Before `first\n    second\\\n  third` after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-173

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-173) · [active transcript](../../cases/recovered_pr6_preserves_literal_line_endings_173.case) · [reviewed transcript](recovered_pr6_preserves_literal_line_endings_173.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-> Before `first  	
+> Before `first
 >   second\
 > third` after.
```

```json
{
  "stdin": "> Before `first  \t\n>   second\\\n> third` after.\n",
  "reviewed_stdout": "> Before `first  \t\n>   second\\\n> third` after.\n",
  "current_stdout": "> Before `first\n>   second\\\n> third` after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-174

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-174)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-Before `first  	
+Before `first
   second\
 third` after.
```

```json
{
  "stdin": "Before `first  \t\r\n  second\\\r\nthird` after.\r\n",
  "reviewed_stdout": "Before `first  \t\r\n  second\\\r\nthird` after.\r\n",
  "current_stdout": "Before `first\r\n  second\\\r\nthird` after.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-175

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-175)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-- Before `first  	
+- Before `first
     second\
   third` after.
```

```json
{
  "stdin": "- Before `first  \t\r\n    second\\\r\n  third` after.\r\n",
  "reviewed_stdout": "- Before `first  \t\r\n    second\\\r\n  third` after.\r\n",
  "current_stdout": "- Before `first\r\n    second\\\r\n  third` after.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-176

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-176)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-> Before `first  	
+> Before `first
 >   second\
 > third` after.
```

```json
{
  "stdin": "> Before `first  \t\r\n>   second\\\r\n> third` after.\r\n",
  "reviewed_stdout": "> Before `first  \t\r\n>   second\\\r\n> third` after.\r\n",
  "current_stdout": "> Before `first\r\n>   second\\\r\n> third` after.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-177

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-177)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-Before `first  	+Before `first   second\ third` after.
```

```json
{
  "stdin": "Before `first  \t\r  second\\\rthird` after.\r",
  "reviewed_stdout": "Before `first  \t\r  second\\\rthird` after.\r",
  "current_stdout": "Before `first\r  second\\\rthird` after.\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-178

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-178)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-- Before `first  	+- Before `first     second\   third` after.
```

```json
{
  "stdin": "- Before `first  \t\r    second\\\r  third` after.\r",
  "reviewed_stdout": "- Before `first  \t\r    second\\\r  third` after.\r",
  "current_stdout": "- Before `first\r    second\\\r  third` after.\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-179

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-179)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-> Before `first  	+> Before `first >   second\ > third` after.
```

```json
{
  "stdin": "> Before `first  \t\r>   second\\\r> third` after.\r",
  "reviewed_stdout": "> Before `first  \t\r>   second\\\r> third` after.\r",
  "current_stdout": "> Before `first\r>   second\\\r> third` after.\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-210

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-210) · [active transcript](../../cases/recovered_pr6_canonical_emphasis_skips_literal_delimiters_210.case) · [reviewed transcript](recovered_pr6_canonical_emphasis_skips_literal_delimiters_210.case)

```text
format --stdin-file-path input.md --wrap sentence --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-# *Before `_a_ [x]( url )` after*
+# *Before `*a_ [x]( url )` after_
```

```json
{
  "stdin": "# _Before `_a_ [x]( url )` after_\n",
  "reviewed_stdout": "# *Before `_a_ [x]( url )` after*\n",
  "current_stdout": "# *Before `*a_ [x]( url )` after_\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-212

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-212) · [active transcript](../../cases/recovered_pr6_canonical_emphasis_skips_literal_delimiters_212.case) · [reviewed transcript](recovered_pr6_canonical_emphasis_skips_literal_delimiters_212.case)

```text
format --stdin-file-path input.md --wrap sentence --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-# *Before $x + _a_ [x]( url )$ after*
+# *Before $x + _a* [x]( url )$ after_
```

```json
{
  "stdin": "# _Before $x + _a_ [x]( url )$ after_\n",
  "reviewed_stdout": "# *Before $x + _a_ [x]( url )$ after*\n",
  "current_stdout": "# *Before $x + _a* [x]( url )$ after_\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-214

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-214) · [active transcript](../../cases/recovered_pr6_preserves_definition_and_footnote_literals_214.case) · [reviewed transcript](recovered_pr6_preserves_definition_and_footnote_literals_214.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
-: Before `first  
-      second [x]( url )` after *outside*.
+: Before `first
+      second [x]( url )` after _outside_.
```

```json
{
  "stdin": "Term\n: Before `first  \n      second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "Term\n: Before `first  \n      second [x]( url )` after *outside*.\n",
  "current_stdout": "Term\n: Before `first\n      second [x]( url )` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-215

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-215)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
-: Before `first  
+: Before `first
       second [x]( url )` after *outside*.
```

```json
{
  "stdin": "Term\n: Before `first  \n      second [x]( url )` after *outside*.\n",
  "reviewed_stdout": "Term\n: Before `first  \n      second [x]( url )` after *outside*.\n",
  "current_stdout": "Term\n: Before `first\n      second [x]( url )` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-216

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-216) · [active transcript](../../cases/recovered_pr6_preserves_definition_and_footnote_literals_216.case) · [reviewed transcript](recovered_pr6_preserves_definition_and_footnote_literals_216.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-[^note]: Before `first  
-    second [x]( url )` after *outside*.
+[^note]: Before `first
+    second [x]( url )` after _outside_.
```

```json
{
  "stdin": "[^note]: Before `first  \n    second [x]( url )` after _outside_.\n",
  "reviewed_stdout": "[^note]: Before `first  \n    second [x]( url )` after *outside*.\n",
  "current_stdout": "[^note]: Before `first\n    second [x]( url )` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-217

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-217)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-[^note]: Before `first  
+[^note]: Before `first
     second [x]( url )` after *outside*.
```

```json
{
  "stdin": "[^note]: Before `first  \n    second [x]( url )` after *outside*.\n",
  "reviewed_stdout": "[^note]: Before `first  \n    second [x]( url )` after *outside*.\n",
  "current_stdout": "[^note]: Before `first\n    second [x]( url )` after *outside*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-218

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-218) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_218.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_218.case)

```text
format --stdin-file-path input.md --wrap none
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd>`first  
-  second [x]( url )`</kbd> after _outside_ [real](target).
+Béfore <kbd>`first \
+  second [x]( url )`</kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd>`first  \n  second [x]( url )`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd>`first  \n  second [x]( url )`</kbd> after _outside_ [real](target).\n",
  "current_stdout": "Béfore <kbd>`first \\\n  second [x]( url )`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-219

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-219)

```text
format --stdin-file-path input.md --wrap none
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd>`first  
+Béfore <kbd>`first \
   second [x]( url )`</kbd> after _outside_ [real](target).
```

```json
{
  "stdin": "Béfore <kbd>`first  \n  second [x]( url )`</kbd> after _outside_ [real](target).\n",
  "reviewed_stdout": "Béfore <kbd>`first  \n  second [x]( url )`</kbd> after _outside_ [real](target).\n",
  "current_stdout": "Béfore <kbd>`first \\\n  second [x]( url )`</kbd> after _outside_ [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-220

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-220) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_220.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_220.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd>`first  
-  second [x]( url )`</kbd> after *outside* [real](target).
+Béfore <kbd>`first \
+  second [x]( url )`</kbd> after *outside* [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd>`first  \n  second [x]( url )`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd>`first  \n  second [x]( url )`</kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd>`first \\\n  second [x]( url )`</kbd> after *outside* [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-221

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-221)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd>`first  
+Béfore <kbd>`first \
   second [x]( url )`</kbd> after *outside* [real](target).
```

```json
{
  "stdin": "Béfore <kbd>`first  \n  second [x]( url )`</kbd> after *outside* [real](target).\n",
  "reviewed_stdout": "Béfore <kbd>`first  \n  second [x]( url )`</kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd>`first \\\n  second [x]( url )`</kbd> after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-222

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-222) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_222.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_222.case)

```text
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd>`first  
-  second [x]( url )`</kbd> after _outside_ [real](target).
+Béfore <kbd>`first
+  second [x]( url )`</kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd>`first  \n  second [x]( url )`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd>`first  \n  second [x]( url )`</kbd> after _outside_ [real](target).\n",
  "current_stdout": "Béfore <kbd>`first\n  second [x]( url )`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-223

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-223)

```text
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd>`first  
+Béfore <kbd>`first
   second [x]( url )`</kbd> after _outside_ [real](target).
```

```json
{
  "stdin": "Béfore <kbd>`first  \n  second [x]( url )`</kbd> after _outside_ [real](target).\n",
  "reviewed_stdout": "Béfore <kbd>`first  \n  second [x]( url )`</kbd> after _outside_ [real](target).\n",
  "current_stdout": "Béfore <kbd>`first\n  second [x]( url )`</kbd> after _outside_ [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-224

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-224) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_224.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_224.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd>`first  
-  second [x]( url )`</kbd> after *outside* [real](target).
+Béfore <kbd>`first
+  second [x]( url )`</kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd>`first  \n  second [x]( url )`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd>`first  \n  second [x]( url )`</kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd>`first\n  second [x]( url )`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-225

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-225)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd>`first  
+Béfore <kbd>`first
   second [x]( url )`</kbd> after *outside* [real](target).
```

```json
{
  "stdin": "Béfore <kbd>`first  \n  second [x]( url )`</kbd> after *outside* [real](target).\n",
  "reviewed_stdout": "Béfore <kbd>`first  \n  second [x]( url )`</kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd>`first\n  second [x]( url )`</kbd> after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-226

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-226) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_226.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_226.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Béfore <kbd>`first  
-    second [x]( url )`</kbd> after _outside_ [real](target).
+- Béfore <kbd>`first
+    second [x]( url )`</kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- Béfore <kbd>`first  \n    second [x]( url )`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "- Béfore <kbd>`first  \n    second [x]( url )`</kbd> after _outside_ [real](target).\n",
  "current_stdout": "- Béfore <kbd>`first\n    second [x]( url )`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-227

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-227)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Béfore <kbd>`first  
+- Béfore <kbd>`first
     second [x]( url )`</kbd> after _outside_ [real](target).
```

```json
{
  "stdin": "- Béfore <kbd>`first  \n    second [x]( url )`</kbd> after _outside_ [real](target).\n",
  "reviewed_stdout": "- Béfore <kbd>`first  \n    second [x]( url )`</kbd> after _outside_ [real](target).\n",
  "current_stdout": "- Béfore <kbd>`first\n    second [x]( url )`</kbd> after _outside_ [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-228

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-228) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_228.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_228.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Béfore <kbd>`first  
-    second [x]( url )`</kbd> after *outside* [real](target).
+- Béfore <kbd>`first
+    second [x]( url )`</kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- Béfore <kbd>`first  \n    second [x]( url )`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "- Béfore <kbd>`first  \n    second [x]( url )`</kbd> after *outside* [real](target).\n",
  "current_stdout": "- Béfore <kbd>`first\n    second [x]( url )`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-229

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-229)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Béfore <kbd>`first  
+- Béfore <kbd>`first
     second [x]( url )`</kbd> after *outside* [real](target).
```

```json
{
  "stdin": "- Béfore <kbd>`first  \n    second [x]( url )`</kbd> after *outside* [real](target).\n",
  "reviewed_stdout": "- Béfore <kbd>`first  \n    second [x]( url )`</kbd> after *outside* [real](target).\n",
  "current_stdout": "- Béfore <kbd>`first\n    second [x]( url )`</kbd> after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-230

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-230) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_230.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_230.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> Béfore <kbd>`first  
->   second [x]( url )`</kbd> after _outside_ [real](target).
+> Béfore <kbd>`first
+>   second [x]( url )`</kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "> Béfore <kbd>`first  \n>   second [x]( url )`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "> Béfore <kbd>`first  \n>   second [x]( url )`</kbd> after _outside_ [real](target).\n",
  "current_stdout": "> Béfore <kbd>`first\n>   second [x]( url )`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-231

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-231)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> Béfore <kbd>`first  
+> Béfore <kbd>`first
 >   second [x]( url )`</kbd> after _outside_ [real](target).
```

```json
{
  "stdin": "> Béfore <kbd>`first  \n>   second [x]( url )`</kbd> after _outside_ [real](target).\n",
  "reviewed_stdout": "> Béfore <kbd>`first  \n>   second [x]( url )`</kbd> after _outside_ [real](target).\n",
  "current_stdout": "> Béfore <kbd>`first\n>   second [x]( url )`</kbd> after _outside_ [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-232

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-232) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_232.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_232.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> Béfore <kbd>`first  
->   second [x]( url )`</kbd> after *outside* [real](target).
+> Béfore <kbd>`first
+>   second [x]( url )`</kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "> Béfore <kbd>`first  \n>   second [x]( url )`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "> Béfore <kbd>`first  \n>   second [x]( url )`</kbd> after *outside* [real](target).\n",
  "current_stdout": "> Béfore <kbd>`first\n>   second [x]( url )`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-233

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-233)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> Béfore <kbd>`first  
+> Béfore <kbd>`first
 >   second [x]( url )`</kbd> after *outside* [real](target).
```

```json
{
  "stdin": "> Béfore <kbd>`first  \n>   second [x]( url )`</kbd> after *outside* [real](target).\n",
  "reviewed_stdout": "> Béfore <kbd>`first  \n>   second [x]( url )`</kbd> after *outside* [real](target).\n",
  "current_stdout": "> Béfore <kbd>`first\n>   second [x]( url )`</kbd> after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-234

Only outside link padding differs: the raw-angle stop in merged #8 intentionally skips normalization.

[Mapping](mapping.md#pr6-234) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_234.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_234.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 Béfore <kbd>$first\
-  second [x]( url )$</kbd> after _outside_ [real](target).
+  second [x]( url )$</kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd>$first\\\n  second [x]( url )$</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd>$first\\\n  second [x]( url )$</kbd> after _outside_ [real](target).\n",
  "current_stdout": "Béfore <kbd>$first\\\n  second [x]( url )$</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-236

Only outside link padding differs: the raw-angle stop in merged #8 intentionally skips normalization.

[Mapping](mapping.md#pr6-236) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_236.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_236.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 Béfore <kbd>$first\
-  second [x]( url )$</kbd> after *outside* [real](target).
+  second [x]( url )$</kbd> after *outside* [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd>$first\\\n  second [x]( url )$</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd>$first\\\n  second [x]( url )$</kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd>$first\\\n  second [x]( url )$</kbd> after *outside* [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-238

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-238) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_238.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_238.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 Béfore <kbd>$first\
-  second [x]( url )$</kbd> after *outside* [real](target).
+  second [x]( url )$</kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd>$first\\\n  second [x]( url )$</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd>$first\\\n  second [x]( url )$</kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd>$first\\\n  second [x]( url )$</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-239

Only outside link padding differs: the raw-angle stop in merged #8 intentionally skips normalization.

[Mapping](mapping.md#pr6-239) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_239.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_239.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 - Béfore <kbd>$first\
-    second [x]( url )$</kbd> after _outside_ [real](target).
+    second [x]( url )$</kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- Béfore <kbd>$first\\\n    second [x]( url )$</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "- Béfore <kbd>$first\\\n    second [x]( url )$</kbd> after _outside_ [real](target).\n",
  "current_stdout": "- Béfore <kbd>$first\\\n    second [x]( url )$</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-241

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-241) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_241.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_241.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 - Béfore <kbd>$first\
-    second [x]( url )$</kbd> after *outside* [real](target).
+    second [x]( url )$</kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- Béfore <kbd>$first\\\n    second [x]( url )$</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "- Béfore <kbd>$first\\\n    second [x]( url )$</kbd> after *outside* [real](target).\n",
  "current_stdout": "- Béfore <kbd>$first\\\n    second [x]( url )$</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-243

Only outside link padding differs: the raw-angle stop in merged #8 intentionally skips normalization.

[Mapping](mapping.md#pr6-243) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_243.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_243.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 > Béfore <kbd>$first\
->   second [x]( url )$</kbd> after _outside_ [real](target).
+>   second [x]( url )$</kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "> Béfore <kbd>$first\\\n>   second [x]( url )$</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "> Béfore <kbd>$first\\\n>   second [x]( url )$</kbd> after _outside_ [real](target).\n",
  "current_stdout": "> Béfore <kbd>$first\\\n>   second [x]( url )$</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-245

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-245) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_245.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_245.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 > Béfore <kbd>$first\
->   second [x]( url )$</kbd> after *outside* [real](target).
+>   second [x]( url )$</kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "> Béfore <kbd>$first\\\n>   second [x]( url )$</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "> Béfore <kbd>$first\\\n>   second [x]( url )$</kbd> after *outside* [real](target).\n",
  "current_stdout": "> Béfore <kbd>$first\\\n>   second [x]( url )$</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-247

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-247) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_247.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_247.case)

```text
format --stdin-file-path input.md --wrap none
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd>`first	
-  second {{< include file >}}`</kbd> after _outside_ [real](target).
+Béfore <kbd>`first
+  second {{< include file >}}`</kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd>`first\t\n  second {{< include file >}}`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd>`first\t\n  second {{< include file >}}`</kbd> after _outside_ [real](target).\n",
  "current_stdout": "Béfore <kbd>`first\n  second {{< include file >}}`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-248

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-248)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd>`first	
+Béfore <kbd>`first
   second {{< include file >}}`</kbd> after _outside_ [real](target).
```

```json
{
  "stdin": "Béfore <kbd>`first\t\n  second {{< include file >}}`</kbd> after _outside_ [real](target).\n",
  "reviewed_stdout": "Béfore <kbd>`first\t\n  second {{< include file >}}`</kbd> after _outside_ [real](target).\n",
  "current_stdout": "Béfore <kbd>`first\n  second {{< include file >}}`</kbd> after _outside_ [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-249

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-249) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_249.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_249.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd>`first	
-  second {{< include file >}}`</kbd> after *outside* [real](target).
+Béfore <kbd>`first
+  second {{< include file >}}`</kbd> after *outside* [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd>`first\t\n  second {{< include file >}}`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd>`first\t\n  second {{< include file >}}`</kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd>`first\n  second {{< include file >}}`</kbd> after *outside* [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-250

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-250)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd>`first	
+Béfore <kbd>`first
   second {{< include file >}}`</kbd> after *outside* [real](target).
```

```json
{
  "stdin": "Béfore <kbd>`first\t\n  second {{< include file >}}`</kbd> after *outside* [real](target).\n",
  "reviewed_stdout": "Béfore <kbd>`first\t\n  second {{< include file >}}`</kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd>`first\n  second {{< include file >}}`</kbd> after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-251

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-251) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_251.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_251.case)

```text
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd>`first	
-  second {{< include file >}}`</kbd> after _outside_ [real](target).
+Béfore <kbd>`first
+  second {{< include file >}}`</kbd> after _outside_ [real]( target ).
```

```json
{
  "stdin": "Béfore <kbd>`first\t\n  second {{< include file >}}`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd>`first\t\n  second {{< include file >}}`</kbd> after _outside_ [real](target).\n",
  "current_stdout": "Béfore <kbd>`first\n  second {{< include file >}}`</kbd> after _outside_ [real]( target ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-252

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-252) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_252.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_252.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd>`first	
-  second {{< include file >}}`</kbd> after *outside* [real](target).
+Béfore <kbd>`first
+  second {{< include file >}}`</kbd> after *outside* [real]( target ).
```

```json
{
  "stdin": "Béfore <kbd>`first\t\n  second {{< include file >}}`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd>`first\t\n  second {{< include file >}}`</kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd>`first\n  second {{< include file >}}`</kbd> after *outside* [real]( target ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-253

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-253) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_253.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_253.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Béfore <kbd>`first	
-    second {{< include file >}}`</kbd> after _outside_ [real](target).
+- Béfore <kbd>`first
+    second {{< include file >}}`</kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- Béfore <kbd>`first\t\n    second {{< include file >}}`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "- Béfore <kbd>`first\t\n    second {{< include file >}}`</kbd> after _outside_ [real](target).\n",
  "current_stdout": "- Béfore <kbd>`first\n    second {{< include file >}}`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-254

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-254)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Béfore <kbd>`first	
+- Béfore <kbd>`first
     second {{< include file >}}`</kbd> after _outside_ [real](target).
```

```json
{
  "stdin": "- Béfore <kbd>`first\t\n    second {{< include file >}}`</kbd> after _outside_ [real](target).\n",
  "reviewed_stdout": "- Béfore <kbd>`first\t\n    second {{< include file >}}`</kbd> after _outside_ [real](target).\n",
  "current_stdout": "- Béfore <kbd>`first\n    second {{< include file >}}`</kbd> after _outside_ [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-255

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-255) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_255.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_255.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Béfore <kbd>`first	
-    second {{< include file >}}`</kbd> after *outside* [real](target).
+- Béfore <kbd>`first
+    second {{< include file >}}`</kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- Béfore <kbd>`first\t\n    second {{< include file >}}`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "- Béfore <kbd>`first\t\n    second {{< include file >}}`</kbd> after *outside* [real](target).\n",
  "current_stdout": "- Béfore <kbd>`first\n    second {{< include file >}}`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-256

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-256)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Béfore <kbd>`first	
+- Béfore <kbd>`first
     second {{< include file >}}`</kbd> after *outside* [real](target).
```

```json
{
  "stdin": "- Béfore <kbd>`first\t\n    second {{< include file >}}`</kbd> after *outside* [real](target).\n",
  "reviewed_stdout": "- Béfore <kbd>`first\t\n    second {{< include file >}}`</kbd> after *outside* [real](target).\n",
  "current_stdout": "- Béfore <kbd>`first\n    second {{< include file >}}`</kbd> after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-257

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-257) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_257.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_257.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> Béfore <kbd>`first	
->   second {{< include file >}}`</kbd> after _outside_ [real](target).
+> Béfore <kbd>`first second {{< include file >}}`</kbd> after _outside_ [real]( target ).
```

```json
{
  "stdin": "> Béfore <kbd>`first\t\n>   second {{< include file >}}`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "> Béfore <kbd>`first\t\n>   second {{< include file >}}`</kbd> after _outside_ [real](target).\n",
  "current_stdout": "> Béfore <kbd>`first second {{< include file >}}`</kbd> after _outside_ [real]( target ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-258

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-258)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> Béfore <kbd>`first	
->   second {{< include file >}}`</kbd> after _outside_ [real](target).
+> Béfore <kbd>`first second {{< include file >}}`</kbd> after _outside_ [real](target).
```

```json
{
  "stdin": "> Béfore <kbd>`first\t\n>   second {{< include file >}}`</kbd> after _outside_ [real](target).\n",
  "reviewed_stdout": "> Béfore <kbd>`first\t\n>   second {{< include file >}}`</kbd> after _outside_ [real](target).\n",
  "current_stdout": "> Béfore <kbd>`first second {{< include file >}}`</kbd> after _outside_ [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-259

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-259) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_259.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_259.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> Béfore <kbd>`first	
->   second {{< include file >}}`</kbd> after *outside* [real](target).
+> Béfore <kbd>`first second {{< include file >}}`</kbd> after *outside* [real]( target ).
```

```json
{
  "stdin": "> Béfore <kbd>`first\t\n>   second {{< include file >}}`</kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "> Béfore <kbd>`first\t\n>   second {{< include file >}}`</kbd> after *outside* [real](target).\n",
  "current_stdout": "> Béfore <kbd>`first second {{< include file >}}`</kbd> after *outside* [real]( target ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-260

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-260)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> Béfore <kbd>`first	
->   second {{< include file >}}`</kbd> after *outside* [real](target).
+> Béfore <kbd>`first second {{< include file >}}`</kbd> after *outside* [real](target).
```

```json
{
  "stdin": "> Béfore <kbd>`first\t\n>   second {{< include file >}}`</kbd> after *outside* [real](target).\n",
  "reviewed_stdout": "> Béfore <kbd>`first\t\n>   second {{< include file >}}`</kbd> after *outside* [real](target).\n",
  "current_stdout": "> Béfore <kbd>`first second {{< include file >}}`</kbd> after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-261

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-261) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_261.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_261.case)

```text
format --stdin-file-path input.md --wrap none
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd><span>`first  
-  second [x]( url )`</span></kbd> after _outside_ [real](target).
+Béfore <kbd><span>`first \
+  second [x]( url )`</span></kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd><span>`first  \n  second [x]( url )`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd><span>`first  \n  second [x]( url )`</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "Béfore <kbd><span>`first \\\n  second [x]( url )`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-262

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-262)

```text
format --stdin-file-path input.md --wrap none
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd><span>`first  
+Béfore <kbd><span>`first \
   second [x]( url )`</span></kbd> after _outside_ [real](target).
```

```json
{
  "stdin": "Béfore <kbd><span>`first  \n  second [x]( url )`</span></kbd> after _outside_ [real](target).\n",
  "reviewed_stdout": "Béfore <kbd><span>`first  \n  second [x]( url )`</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "Béfore <kbd><span>`first \\\n  second [x]( url )`</span></kbd> after _outside_ [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-263

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-263) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_263.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_263.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd><span>`first  
-  second [x]( url )`</span></kbd> after *outside* [real](target).
+Béfore <kbd><span>`first \
+  second [x]( url )`</span></kbd> after *outside* [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd><span>`first  \n  second [x]( url )`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd><span>`first  \n  second [x]( url )`</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd><span>`first \\\n  second [x]( url )`</span></kbd> after *outside* [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-264

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-264)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd><span>`first  
+Béfore <kbd><span>`first \
   second [x]( url )`</span></kbd> after *outside* [real](target).
```

```json
{
  "stdin": "Béfore <kbd><span>`first  \n  second [x]( url )`</span></kbd> after *outside* [real](target).\n",
  "reviewed_stdout": "Béfore <kbd><span>`first  \n  second [x]( url )`</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd><span>`first \\\n  second [x]( url )`</span></kbd> after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-265

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-265) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_265.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_265.case)

```text
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd><span>`first  
-  second [x]( url )`</span></kbd> after _outside_ [real](target).
+Béfore <kbd><span>`first
+  second [x]( url )`</span></kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd><span>`first  \n  second [x]( url )`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd><span>`first  \n  second [x]( url )`</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "Béfore <kbd><span>`first\n  second [x]( url )`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-266

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-266)

```text
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd><span>`first  
+Béfore <kbd><span>`first
   second [x]( url )`</span></kbd> after _outside_ [real](target).
```

```json
{
  "stdin": "Béfore <kbd><span>`first  \n  second [x]( url )`</span></kbd> after _outside_ [real](target).\n",
  "reviewed_stdout": "Béfore <kbd><span>`first  \n  second [x]( url )`</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "Béfore <kbd><span>`first\n  second [x]( url )`</span></kbd> after _outside_ [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-267

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-267) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_267.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_267.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd><span>`first  
-  second [x]( url )`</span></kbd> after *outside* [real](target).
+Béfore <kbd><span>`first
+  second [x]( url )`</span></kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd><span>`first  \n  second [x]( url )`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd><span>`first  \n  second [x]( url )`</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd><span>`first\n  second [x]( url )`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-268

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-268)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd><span>`first  
+Béfore <kbd><span>`first
   second [x]( url )`</span></kbd> after *outside* [real](target).
```

```json
{
  "stdin": "Béfore <kbd><span>`first  \n  second [x]( url )`</span></kbd> after *outside* [real](target).\n",
  "reviewed_stdout": "Béfore <kbd><span>`first  \n  second [x]( url )`</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd><span>`first\n  second [x]( url )`</span></kbd> after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-269

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-269) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_269.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_269.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Béfore <kbd><span>`first  
-    second [x]( url )`</span></kbd> after _outside_ [real](target).
+- Béfore <kbd><span>`first
+    second [x]( url )`</span></kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- Béfore <kbd><span>`first  \n    second [x]( url )`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "- Béfore <kbd><span>`first  \n    second [x]( url )`</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "- Béfore <kbd><span>`first\n    second [x]( url )`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-270

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-270)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Béfore <kbd><span>`first  
+- Béfore <kbd><span>`first
     second [x]( url )`</span></kbd> after _outside_ [real](target).
```

```json
{
  "stdin": "- Béfore <kbd><span>`first  \n    second [x]( url )`</span></kbd> after _outside_ [real](target).\n",
  "reviewed_stdout": "- Béfore <kbd><span>`first  \n    second [x]( url )`</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "- Béfore <kbd><span>`first\n    second [x]( url )`</span></kbd> after _outside_ [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-271

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-271) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_271.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_271.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Béfore <kbd><span>`first  
-    second [x]( url )`</span></kbd> after *outside* [real](target).
+- Béfore <kbd><span>`first
+    second [x]( url )`</span></kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- Béfore <kbd><span>`first  \n    second [x]( url )`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "- Béfore <kbd><span>`first  \n    second [x]( url )`</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "- Béfore <kbd><span>`first\n    second [x]( url )`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-272

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-272)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Béfore <kbd><span>`first  
+- Béfore <kbd><span>`first
     second [x]( url )`</span></kbd> after *outside* [real](target).
```

```json
{
  "stdin": "- Béfore <kbd><span>`first  \n    second [x]( url )`</span></kbd> after *outside* [real](target).\n",
  "reviewed_stdout": "- Béfore <kbd><span>`first  \n    second [x]( url )`</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "- Béfore <kbd><span>`first\n    second [x]( url )`</span></kbd> after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-273

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-273) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_273.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_273.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> Béfore <kbd><span>`first  
->   second [x]( url )`</span></kbd> after _outside_ [real](target).
+> Béfore <kbd><span>`first
+>   second [x]( url )`</span></kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "> Béfore <kbd><span>`first  \n>   second [x]( url )`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "> Béfore <kbd><span>`first  \n>   second [x]( url )`</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "> Béfore <kbd><span>`first\n>   second [x]( url )`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-274

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-274)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> Béfore <kbd><span>`first  
+> Béfore <kbd><span>`first
 >   second [x]( url )`</span></kbd> after _outside_ [real](target).
```

```json
{
  "stdin": "> Béfore <kbd><span>`first  \n>   second [x]( url )`</span></kbd> after _outside_ [real](target).\n",
  "reviewed_stdout": "> Béfore <kbd><span>`first  \n>   second [x]( url )`</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "> Béfore <kbd><span>`first\n>   second [x]( url )`</span></kbd> after _outside_ [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-275

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-275) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_275.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_275.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> Béfore <kbd><span>`first  
->   second [x]( url )`</span></kbd> after *outside* [real](target).
+> Béfore <kbd><span>`first
+>   second [x]( url )`</span></kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "> Béfore <kbd><span>`first  \n>   second [x]( url )`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "> Béfore <kbd><span>`first  \n>   second [x]( url )`</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "> Béfore <kbd><span>`first\n>   second [x]( url )`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-276

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-276)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-> Béfore <kbd><span>`first  
+> Béfore <kbd><span>`first
 >   second [x]( url )`</span></kbd> after *outside* [real](target).
```

```json
{
  "stdin": "> Béfore <kbd><span>`first  \n>   second [x]( url )`</span></kbd> after *outside* [real](target).\n",
  "reviewed_stdout": "> Béfore <kbd><span>`first  \n>   second [x]( url )`</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "> Béfore <kbd><span>`first\n>   second [x]( url )`</span></kbd> after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-277

Only outside link padding differs: the raw-angle stop in merged #8 intentionally skips normalization.

[Mapping](mapping.md#pr6-277) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_277.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_277.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 Béfore <kbd><span>$first\
-  second [x]( url )$</span></kbd> after _outside_ [real](target).
+  second [x]( url )$</span></kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd><span>$first\\\n  second [x]( url )$</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd><span>$first\\\n  second [x]( url )$</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "Béfore <kbd><span>$first\\\n  second [x]( url )$</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-279

Only outside link padding differs: the raw-angle stop in merged #8 intentionally skips normalization.

[Mapping](mapping.md#pr6-279) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_279.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_279.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 Béfore <kbd><span>$first\
-  second [x]( url )$</span></kbd> after *outside* [real](target).
+  second [x]( url )$</span></kbd> after *outside* [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd><span>$first\\\n  second [x]( url )$</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd><span>$first\\\n  second [x]( url )$</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd><span>$first\\\n  second [x]( url )$</span></kbd> after *outside* [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-281

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-281) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_281.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_281.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 Béfore <kbd><span>$first\
-  second [x]( url )$</span></kbd> after *outside* [real](target).
+  second [x]( url )$</span></kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd><span>$first\\\n  second [x]( url )$</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd><span>$first\\\n  second [x]( url )$</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd><span>$first\\\n  second [x]( url )$</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-282

Only outside link padding differs: the raw-angle stop in merged #8 intentionally skips normalization.

[Mapping](mapping.md#pr6-282) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_282.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_282.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 - Béfore <kbd><span>$first\
-    second [x]( url )$</span></kbd> after _outside_ [real](target).
+    second [x]( url )$</span></kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- Béfore <kbd><span>$first\\\n    second [x]( url )$</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "- Béfore <kbd><span>$first\\\n    second [x]( url )$</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "- Béfore <kbd><span>$first\\\n    second [x]( url )$</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-284

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-284) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_284.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_284.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 - Béfore <kbd><span>$first\
-    second [x]( url )$</span></kbd> after *outside* [real](target).
+    second [x]( url )$</span></kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- Béfore <kbd><span>$first\\\n    second [x]( url )$</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "- Béfore <kbd><span>$first\\\n    second [x]( url )$</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "- Béfore <kbd><span>$first\\\n    second [x]( url )$</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-286

Only outside link padding differs: the raw-angle stop in merged #8 intentionally skips normalization.

[Mapping](mapping.md#pr6-286) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_286.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_286.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 > Béfore <kbd><span>$first\
->   second [x]( url )$</span></kbd> after _outside_ [real](target).
+>   second [x]( url )$</span></kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "> Béfore <kbd><span>$first\\\n>   second [x]( url )$</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "> Béfore <kbd><span>$first\\\n>   second [x]( url )$</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "> Béfore <kbd><span>$first\\\n>   second [x]( url )$</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-288

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-288) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_288.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_288.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 > Béfore <kbd><span>$first\
->   second [x]( url )$</span></kbd> after *outside* [real](target).
+>   second [x]( url )$</span></kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "> Béfore <kbd><span>$first\\\n>   second [x]( url )$</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "> Béfore <kbd><span>$first\\\n>   second [x]( url )$</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "> Béfore <kbd><span>$first\\\n>   second [x]( url )$</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-290

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-290) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_290.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_290.case)

```text
format --stdin-file-path input.md --wrap none
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd><span>`first	
-  second {{< include file >}}`</span></kbd> after _outside_ [real](target).
+Béfore <kbd><span>`first
+  second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd><span>`first\t\n  second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd><span>`first\t\n  second {{< include file >}}`</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "Béfore <kbd><span>`first\n  second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-291

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-291)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd><span>`first	
+Béfore <kbd><span>`first
   second {{< include file >}}`</span></kbd> after _outside_ [real](target).
```

```json
{
  "stdin": "Béfore <kbd><span>`first\t\n  second {{< include file >}}`</span></kbd> after _outside_ [real](target).\n",
  "reviewed_stdout": "Béfore <kbd><span>`first\t\n  second {{< include file >}}`</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "Béfore <kbd><span>`first\n  second {{< include file >}}`</span></kbd> after _outside_ [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-292

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-292) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_292.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_292.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd><span>`first	
-  second {{< include file >}}`</span></kbd> after *outside* [real](target).
+Béfore <kbd><span>`first
+  second {{< include file >}}`</span></kbd> after *outside* [real](  target  ).
```

```json
{
  "stdin": "Béfore <kbd><span>`first\t\n  second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd><span>`first\t\n  second {{< include file >}}`</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd><span>`first\n  second {{< include file >}}`</span></kbd> after *outside* [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-293

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-293)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd><span>`first	
+Béfore <kbd><span>`first
   second {{< include file >}}`</span></kbd> after *outside* [real](target).
```

```json
{
  "stdin": "Béfore <kbd><span>`first\t\n  second {{< include file >}}`</span></kbd> after *outside* [real](target).\n",
  "reviewed_stdout": "Béfore <kbd><span>`first\t\n  second {{< include file >}}`</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd><span>`first\n  second {{< include file >}}`</span></kbd> after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-294

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-294) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_294.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_294.case)

```text
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd><span>`first	
-  second {{< include file >}}`</span></kbd> after _outside_ [real](target).
+Béfore <kbd><span>`first
+  second {{< include file >}}`</span></kbd> after _outside_ [real]( target ).
```

```json
{
  "stdin": "Béfore <kbd><span>`first\t\n  second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd><span>`first\t\n  second {{< include file >}}`</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "Béfore <kbd><span>`first\n  second {{< include file >}}`</span></kbd> after _outside_ [real]( target ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-295

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-295) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_295.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_295.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-Béfore <kbd><span>`first	
-  second {{< include file >}}`</span></kbd> after *outside* [real](target).
+Béfore <kbd><span>`first
+  second {{< include file >}}`</span></kbd> after *outside* [real]( target ).
```

```json
{
  "stdin": "Béfore <kbd><span>`first\t\n  second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Béfore <kbd><span>`first\t\n  second {{< include file >}}`</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "Béfore <kbd><span>`first\n  second {{< include file >}}`</span></kbd> after *outside* [real]( target ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-296

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-296) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_296.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_296.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Béfore <kbd><span>`first	
-    second {{< include file >}}`</span></kbd> after _outside_ [real](target).
+- Béfore <kbd><span>`first
+    second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- Béfore <kbd><span>`first\t\n    second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "- Béfore <kbd><span>`first\t\n    second {{< include file >}}`</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "- Béfore <kbd><span>`first\n    second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-297

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-297)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Béfore <kbd><span>`first	
+- Béfore <kbd><span>`first
     second {{< include file >}}`</span></kbd> after _outside_ [real](target).
```

```json
{
  "stdin": "- Béfore <kbd><span>`first\t\n    second {{< include file >}}`</span></kbd> after _outside_ [real](target).\n",
  "reviewed_stdout": "- Béfore <kbd><span>`first\t\n    second {{< include file >}}`</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "- Béfore <kbd><span>`first\n    second {{< include file >}}`</span></kbd> after _outside_ [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-298

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-298) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_298.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_298.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Béfore <kbd><span>`first	
-    second {{< include file >}}`</span></kbd> after *outside* [real](target).
+- Béfore <kbd><span>`first
+    second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- Béfore <kbd><span>`first\t\n    second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "- Béfore <kbd><span>`first\t\n    second {{< include file >}}`</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "- Béfore <kbd><span>`first\n    second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-299

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-299)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- Béfore <kbd><span>`first	
+- Béfore <kbd><span>`first
     second {{< include file >}}`</span></kbd> after *outside* [real](target).
```

```json
{
  "stdin": "- Béfore <kbd><span>`first\t\n    second {{< include file >}}`</span></kbd> after *outside* [real](target).\n",
  "reviewed_stdout": "- Béfore <kbd><span>`first\t\n    second {{< include file >}}`</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "- Béfore <kbd><span>`first\n    second {{< include file >}}`</span></kbd> after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-300

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-300) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_300.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_300.case)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> Béfore <kbd><span>`first	
->   second {{< include file >}}`</span></kbd> after _outside_ [real](target).
+> Béfore <kbd><span>`first second {{< include file >}}`</span></kbd> after _outside_ [real]( target ).
```

```json
{
  "stdin": "> Béfore <kbd><span>`first\t\n>   second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "> Béfore <kbd><span>`first\t\n>   second {{< include file >}}`</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "> Béfore <kbd><span>`first second {{< include file >}}`</span></kbd> after _outside_ [real]( target ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-301

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-301)

```text
format --stdin-file-path input.md --wrap none
format --stdin-file-path input.md --wrap paragraph
format --stdin-file-path input.md --wrap sentence
format --stdin-file-path input.md --wrap 120
format --stdin-file-path input.md --wrap sentence:120
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> Béfore <kbd><span>`first	
->   second {{< include file >}}`</span></kbd> after _outside_ [real](target).
+> Béfore <kbd><span>`first second {{< include file >}}`</span></kbd> after _outside_ [real](target).
```

```json
{
  "stdin": "> Béfore <kbd><span>`first\t\n>   second {{< include file >}}`</span></kbd> after _outside_ [real](target).\n",
  "reviewed_stdout": "> Béfore <kbd><span>`first\t\n>   second {{< include file >}}`</span></kbd> after _outside_ [real](target).\n",
  "current_stdout": "> Béfore <kbd><span>`first second {{< include file >}}`</span></kbd> after _outside_ [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-302

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-302) · [active transcript](../../cases/recovered_pr6_preserves_multiline_literals_nested_in_markup_302.case) · [reviewed transcript](recovered_pr6_preserves_multiline_literals_nested_in_markup_302.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> Béfore <kbd><span>`first	
->   second {{< include file >}}`</span></kbd> after *outside* [real](target).
+> Béfore <kbd><span>`first second {{< include file >}}`</span></kbd> after *outside* [real]( target ).
```

```json
{
  "stdin": "> Béfore <kbd><span>`first\t\n>   second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "> Béfore <kbd><span>`first\t\n>   second {{< include file >}}`</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "> Béfore <kbd><span>`first second {{< include file >}}`</span></kbd> after *outside* [real]( target ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-303

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-303)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-> Béfore <kbd><span>`first	
->   second {{< include file >}}`</span></kbd> after *outside* [real](target).
+> Béfore <kbd><span>`first second {{< include file >}}`</span></kbd> after *outside* [real](target).
```

```json
{
  "stdin": "> Béfore <kbd><span>`first\t\n>   second {{< include file >}}`</span></kbd> after *outside* [real](target).\n",
  "reviewed_stdout": "> Béfore <kbd><span>`first\t\n>   second {{< include file >}}`</span></kbd> after *outside* [real](target).\n",
  "current_stdout": "> Béfore <kbd><span>`first second {{< include file >}}`</span></kbd> after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-586

Vertical-tab handling differs from #6; no merged policy decision for these exact observables was established.

[Mapping](mapping.md#pr6-586) · [active transcript](../../cases/recovered_pr6_keeps_unsupported_whitespace_out_of_reflow_586.case) · [reviewed transcript](recovered_pr6_keeps_unsupported_whitespace_out_of_reflow_586.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 first-second _outside_ [real](  url  ).
+second *outside* [real](url).
```

```json
{
  "stdin": "first\u000bsecond _outside_ [real](  url  ).\n",
  "reviewed_stdout": "first\u000bsecond _outside_ [real](  url  ).\n",
  "current_stdout": "first\u000bsecond *outside* [real](url).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-587

Vertical-tab handling differs from #6; no merged policy decision for these exact observables was established.

[Mapping](mapping.md#pr6-587) · [active transcript](../../cases/recovered_pr6_keeps_unsupported_whitespace_out_of_reflow_587.case) · [reviewed transcript](recovered_pr6_keeps_unsupported_whitespace_out_of_reflow_587.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 first\-second _outside_ [real](  url  ).
+second *outside* [real](url).
```

```json
{
  "stdin": "first\\\u000bsecond _outside_ [real](  url  ).\n",
  "reviewed_stdout": "first\\\u000bsecond _outside_ [real](  url  ).\n",
  "current_stdout": "first\\\u000bsecond *outside* [real](url).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-588

Vertical-tab handling differs from #6; no merged policy decision for these exact observables was established.

[Mapping](mapping.md#pr6-588) · [active transcript](../../cases/recovered_pr6_keeps_unsupported_whitespace_out_of_reflow_588.case) · [reviewed transcript](recovered_pr6_keeps_unsupported_whitespace_out_of_reflow_588.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 first\\-second _outside_ [real](  url  ).
+second *outside* [real](url).
```

```json
{
  "stdin": "first\\\\\u000bsecond _outside_ [real](  url  ).\n",
  "reviewed_stdout": "first\\\\\u000bsecond _outside_ [real](  url  ).\n",
  "current_stdout": "first\\\\\u000bsecond *outside* [real](url).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-601

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-601) · [active transcript](../../cases/recovered_pr6_normalizes_list_separators_without_trimming_literals_601.case) · [reviewed transcript](recovered_pr6_normalizes_list_separators_without_trimming_literals_601.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- Before `first  
-    second [x]( url )` after *outside* [real](target).
+- Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
 
   Next paragraph.
```

```json
{
  "stdin": "- Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n  \n  Next paragraph.\n",
  "reviewed_stdout": "- Before `first  \n    second [x]( url )` after *outside* [real](target).\n\n  Next paragraph.\n",
  "current_stdout": "- Before `first\n    second [x]( url )` after _outside_ [real](  target  ).\n\n  Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-602

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-602)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- Before `first  
+- Before `first
     second [x]( url )` after *outside* [real](target).
 
   Next paragraph.
```

```json
{
  "stdin": "- Before `first  \n    second [x]( url )` after *outside* [real](target).\n\n  Next paragraph.\n",
  "reviewed_stdout": "- Before `first  \n    second [x]( url )` after *outside* [real](target).\n\n  Next paragraph.\n",
  "current_stdout": "- Before `first\n    second [x]( url )` after *outside* [real](target).\n\n  Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-603

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-603) · [active transcript](../../cases/recovered_pr6_normalizes_list_separators_without_trimming_literals_603.case) · [reviewed transcript](recovered_pr6_normalizes_list_separators_without_trimming_literals_603.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- Before `first  
-    second [x]( url )` after *outside* [real](target).
+- Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
 
   Next paragraph.
```

```json
{
  "stdin": "- Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n\t \n  Next paragraph.\n",
  "reviewed_stdout": "- Before `first  \n    second [x]( url )` after *outside* [real](target).\n\n  Next paragraph.\n",
  "current_stdout": "- Before `first\n    second [x]( url )` after _outside_ [real](  target  ).\n\n  Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-604

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-604)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- Before `first  
-    second [x]( url )` after *outside* [real](target).
+- Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
 
   Next paragraph.
```

```json
{
  "stdin": "- Before `first  \r\n    second [x]( url )` after _outside_ [real](  target  ).\r\n  \r\n  Next paragraph.\r\n",
  "reviewed_stdout": "- Before `first  \r\n    second [x]( url )` after *outside* [real](target).\r\n\r\n  Next paragraph.\r\n",
  "current_stdout": "- Before `first\r\n    second [x]( url )` after _outside_ [real](  target  ).\r\n\r\n  Next paragraph.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-605

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-605)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- Before `first  
+- Before `first
     second [x]( url )` after *outside* [real](target).
 
   Next paragraph.
```

```json
{
  "stdin": "- Before `first  \r\n    second [x]( url )` after *outside* [real](target).\r\n\r\n  Next paragraph.\r\n",
  "reviewed_stdout": "- Before `first  \r\n    second [x]( url )` after *outside* [real](target).\r\n\r\n  Next paragraph.\r\n",
  "current_stdout": "- Before `first\r\n    second [x]( url )` after *outside* [real](target).\r\n\r\n  Next paragraph.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-606

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-606)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- Before `first  
-    second [x]( url )` after *outside* [real](target).
+- Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
 
   Next paragraph.
```

```json
{
  "stdin": "- Before `first  \r\n    second [x]( url )` after _outside_ [real](  target  ).\r\n\t \r\n  Next paragraph.\r\n",
  "reviewed_stdout": "- Before `first  \r\n    second [x]( url )` after *outside* [real](target).\r\n\r\n  Next paragraph.\r\n",
  "current_stdout": "- Before `first\r\n    second [x]( url )` after _outside_ [real](  target  ).\r\n\r\n  Next paragraph.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-607

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-607)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- Before `first  -    second [x]( url )` after *outside* [real](target).+- Before `first+    second [x]( url )` after _outside_ [real](  target  ).    Next paragraph.
```

```json
{
  "stdin": "- Before `first  \r    second [x]( url )` after _outside_ [real](  target  ).\r  \r  Next paragraph.\r",
  "reviewed_stdout": "- Before `first  \r    second [x]( url )` after *outside* [real](target).\r\r  Next paragraph.\r",
  "current_stdout": "- Before `first\r    second [x]( url )` after _outside_ [real](  target  ).\r\r  Next paragraph.\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-608

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-608)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- Before `first  +- Before `first     second [x]( url )` after *outside* [real](target).    Next paragraph.
```

```json
{
  "stdin": "- Before `first  \r    second [x]( url )` after *outside* [real](target).\r\r  Next paragraph.\r",
  "reviewed_stdout": "- Before `first  \r    second [x]( url )` after *outside* [real](target).\r\r  Next paragraph.\r",
  "current_stdout": "- Before `first\r    second [x]( url )` after *outside* [real](target).\r\r  Next paragraph.\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-609

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-609)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- Before `first  -    second [x]( url )` after *outside* [real](target).+- Before `first+    second [x]( url )` after _outside_ [real](  target  ).    Next paragraph.
```

```json
{
  "stdin": "- Before `first  \r    second [x]( url )` after _outside_ [real](  target  ).\r\t \r  Next paragraph.\r",
  "reviewed_stdout": "- Before `first  \r    second [x]( url )` after *outside* [real](target).\r\r  Next paragraph.\r",
  "current_stdout": "- Before `first\r    second [x]( url )` after _outside_ [real](  target  ).\r\r  Next paragraph.\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-610

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-610) · [active transcript](../../cases/recovered_pr6_normalizes_list_separators_without_trimming_literals_610.case) · [reviewed transcript](recovered_pr6_normalizes_list_separators_without_trimming_literals_610.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-1. Before `first  
-     second [x]( url )` after *outside* [real](target).
+1. Before `first
+     second [x]( url )` after _outside_ [real](  target  ).
 
    Next paragraph.
```

```json
{
  "stdin": "1. Before `first  \n     second [x]( url )` after _outside_ [real](  target  ).\n  \n   Next paragraph.\n",
  "reviewed_stdout": "1. Before `first  \n     second [x]( url )` after *outside* [real](target).\n\n   Next paragraph.\n",
  "current_stdout": "1. Before `first\n     second [x]( url )` after _outside_ [real](  target  ).\n\n   Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-611

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-611)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-1. Before `first  
+1. Before `first
      second [x]( url )` after *outside* [real](target).
 
    Next paragraph.
```

```json
{
  "stdin": "1. Before `first  \n     second [x]( url )` after *outside* [real](target).\n\n   Next paragraph.\n",
  "reviewed_stdout": "1. Before `first  \n     second [x]( url )` after *outside* [real](target).\n\n   Next paragraph.\n",
  "current_stdout": "1. Before `first\n     second [x]( url )` after *outside* [real](target).\n\n   Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-612

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-612) · [active transcript](../../cases/recovered_pr6_normalizes_list_separators_without_trimming_literals_612.case) · [reviewed transcript](recovered_pr6_normalizes_list_separators_without_trimming_literals_612.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-1. Before `first  
-     second [x]( url )` after *outside* [real](target).
+1. Before `first
+     second [x]( url )` after _outside_ [real](  target  ).
 
    Next paragraph.
```

```json
{
  "stdin": "1. Before `first  \n     second [x]( url )` after _outside_ [real](  target  ).\n\t \n   Next paragraph.\n",
  "reviewed_stdout": "1. Before `first  \n     second [x]( url )` after *outside* [real](target).\n\n   Next paragraph.\n",
  "current_stdout": "1. Before `first\n     second [x]( url )` after _outside_ [real](  target  ).\n\n   Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-613

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-613)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-1. Before `first  
-     second [x]( url )` after *outside* [real](target).
+1. Before `first
+     second [x]( url )` after _outside_ [real](  target  ).
 
    Next paragraph.
```

```json
{
  "stdin": "1. Before `first  \r\n     second [x]( url )` after _outside_ [real](  target  ).\r\n  \r\n   Next paragraph.\r\n",
  "reviewed_stdout": "1. Before `first  \r\n     second [x]( url )` after *outside* [real](target).\r\n\r\n   Next paragraph.\r\n",
  "current_stdout": "1. Before `first\r\n     second [x]( url )` after _outside_ [real](  target  ).\r\n\r\n   Next paragraph.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-614

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-614)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-1. Before `first  
+1. Before `first
      second [x]( url )` after *outside* [real](target).
 
    Next paragraph.
```

```json
{
  "stdin": "1. Before `first  \r\n     second [x]( url )` after *outside* [real](target).\r\n\r\n   Next paragraph.\r\n",
  "reviewed_stdout": "1. Before `first  \r\n     second [x]( url )` after *outside* [real](target).\r\n\r\n   Next paragraph.\r\n",
  "current_stdout": "1. Before `first\r\n     second [x]( url )` after *outside* [real](target).\r\n\r\n   Next paragraph.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-615

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-615)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-1. Before `first  
-     second [x]( url )` after *outside* [real](target).
+1. Before `first
+     second [x]( url )` after _outside_ [real](  target  ).
 
    Next paragraph.
```

```json
{
  "stdin": "1. Before `first  \r\n     second [x]( url )` after _outside_ [real](  target  ).\r\n\t \r\n   Next paragraph.\r\n",
  "reviewed_stdout": "1. Before `first  \r\n     second [x]( url )` after *outside* [real](target).\r\n\r\n   Next paragraph.\r\n",
  "current_stdout": "1. Before `first\r\n     second [x]( url )` after _outside_ [real](  target  ).\r\n\r\n   Next paragraph.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-616

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-616)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-1. Before `first  -     second [x]( url )` after *outside* [real](target).+1. Before `first+     second [x]( url )` after _outside_ [real](  target  ).     Next paragraph.
```

```json
{
  "stdin": "1. Before `first  \r     second [x]( url )` after _outside_ [real](  target  ).\r  \r   Next paragraph.\r",
  "reviewed_stdout": "1. Before `first  \r     second [x]( url )` after *outside* [real](target).\r\r   Next paragraph.\r",
  "current_stdout": "1. Before `first\r     second [x]( url )` after _outside_ [real](  target  ).\r\r   Next paragraph.\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-617

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-617)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-1. Before `first  +1. Before `first      second [x]( url )` after *outside* [real](target).     Next paragraph.
```

```json
{
  "stdin": "1. Before `first  \r     second [x]( url )` after *outside* [real](target).\r\r   Next paragraph.\r",
  "reviewed_stdout": "1. Before `first  \r     second [x]( url )` after *outside* [real](target).\r\r   Next paragraph.\r",
  "current_stdout": "1. Before `first\r     second [x]( url )` after *outside* [real](target).\r\r   Next paragraph.\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-618

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-618)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-1. Before `first  -     second [x]( url )` after *outside* [real](target).+1. Before `first+     second [x]( url )` after _outside_ [real](  target  ).     Next paragraph.
```

```json
{
  "stdin": "1. Before `first  \r     second [x]( url )` after _outside_ [real](  target  ).\r\t \r   Next paragraph.\r",
  "reviewed_stdout": "1. Before `first  \r     second [x]( url )` after *outside* [real](target).\r\r   Next paragraph.\r",
  "current_stdout": "1. Before `first\r     second [x]( url )` after _outside_ [real](  target  ).\r\r   Next paragraph.\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-619

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-619) · [active transcript](../../cases/recovered_pr6_normalizes_list_separators_without_trimming_literals_619.case) · [reviewed transcript](recovered_pr6_normalizes_list_separators_without_trimming_literals_619.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- [x] Before `first  
-        second [x]( url )` after *outside* [real](target).
+- [x] Before `first
+        second [x]( url )` after _outside_ [real](  target  ).
 
       Next paragraph.
```

```json
{
  "stdin": "- [x] Before `first  \n        second [x]( url )` after _outside_ [real](  target  ).\n  \n      Next paragraph.\n",
  "reviewed_stdout": "- [x] Before `first  \n        second [x]( url )` after *outside* [real](target).\n\n      Next paragraph.\n",
  "current_stdout": "- [x] Before `first\n        second [x]( url )` after _outside_ [real](  target  ).\n\n      Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-620

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-620)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- [x] Before `first  
+- [x] Before `first
         second [x]( url )` after *outside* [real](target).
 
       Next paragraph.
```

```json
{
  "stdin": "- [x] Before `first  \n        second [x]( url )` after *outside* [real](target).\n\n      Next paragraph.\n",
  "reviewed_stdout": "- [x] Before `first  \n        second [x]( url )` after *outside* [real](target).\n\n      Next paragraph.\n",
  "current_stdout": "- [x] Before `first\n        second [x]( url )` after *outside* [real](target).\n\n      Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-621

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-621) · [active transcript](../../cases/recovered_pr6_normalizes_list_separators_without_trimming_literals_621.case) · [reviewed transcript](recovered_pr6_normalizes_list_separators_without_trimming_literals_621.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- [x] Before `first  
-        second [x]( url )` after *outside* [real](target).
+- [x] Before `first
+        second [x]( url )` after _outside_ [real](  target  ).
 
       Next paragraph.
```

```json
{
  "stdin": "- [x] Before `first  \n        second [x]( url )` after _outside_ [real](  target  ).\n\t \n      Next paragraph.\n",
  "reviewed_stdout": "- [x] Before `first  \n        second [x]( url )` after *outside* [real](target).\n\n      Next paragraph.\n",
  "current_stdout": "- [x] Before `first\n        second [x]( url )` after _outside_ [real](  target  ).\n\n      Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-622

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-622)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- [x] Before `first  
-        second [x]( url )` after *outside* [real](target).
+- [x] Before `first
+        second [x]( url )` after _outside_ [real](  target  ).
 
       Next paragraph.
```

```json
{
  "stdin": "- [x] Before `first  \r\n        second [x]( url )` after _outside_ [real](  target  ).\r\n  \r\n      Next paragraph.\r\n",
  "reviewed_stdout": "- [x] Before `first  \r\n        second [x]( url )` after *outside* [real](target).\r\n\r\n      Next paragraph.\r\n",
  "current_stdout": "- [x] Before `first\r\n        second [x]( url )` after _outside_ [real](  target  ).\r\n\r\n      Next paragraph.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-623

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-623)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- [x] Before `first  
+- [x] Before `first
         second [x]( url )` after *outside* [real](target).
 
       Next paragraph.
```

```json
{
  "stdin": "- [x] Before `first  \r\n        second [x]( url )` after *outside* [real](target).\r\n\r\n      Next paragraph.\r\n",
  "reviewed_stdout": "- [x] Before `first  \r\n        second [x]( url )` after *outside* [real](target).\r\n\r\n      Next paragraph.\r\n",
  "current_stdout": "- [x] Before `first\r\n        second [x]( url )` after *outside* [real](target).\r\n\r\n      Next paragraph.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-624

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-624)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- [x] Before `first  
-        second [x]( url )` after *outside* [real](target).
+- [x] Before `first
+        second [x]( url )` after _outside_ [real](  target  ).
 
       Next paragraph.
```

```json
{
  "stdin": "- [x] Before `first  \r\n        second [x]( url )` after _outside_ [real](  target  ).\r\n\t \r\n      Next paragraph.\r\n",
  "reviewed_stdout": "- [x] Before `first  \r\n        second [x]( url )` after *outside* [real](target).\r\n\r\n      Next paragraph.\r\n",
  "current_stdout": "- [x] Before `first\r\n        second [x]( url )` after _outside_ [real](  target  ).\r\n\r\n      Next paragraph.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-625

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-625)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- [x] Before `first  -        second [x]( url )` after *outside* [real](target).+- [x] Before `first+        second [x]( url )` after _outside_ [real](  target  ).        Next paragraph.
```

```json
{
  "stdin": "- [x] Before `first  \r        second [x]( url )` after _outside_ [real](  target  ).\r  \r      Next paragraph.\r",
  "reviewed_stdout": "- [x] Before `first  \r        second [x]( url )` after *outside* [real](target).\r\r      Next paragraph.\r",
  "current_stdout": "- [x] Before `first\r        second [x]( url )` after _outside_ [real](  target  ).\r\r      Next paragraph.\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-626

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-626)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- [x] Before `first  +- [x] Before `first         second [x]( url )` after *outside* [real](target).        Next paragraph.
```

```json
{
  "stdin": "- [x] Before `first  \r        second [x]( url )` after *outside* [real](target).\r\r      Next paragraph.\r",
  "reviewed_stdout": "- [x] Before `first  \r        second [x]( url )` after *outside* [real](target).\r\r      Next paragraph.\r",
  "current_stdout": "- [x] Before `first\r        second [x]( url )` after *outside* [real](target).\r\r      Next paragraph.\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-627

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-627)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- [x] Before `first  -        second [x]( url )` after *outside* [real](target).+- [x] Before `first+        second [x]( url )` after _outside_ [real](  target  ).        Next paragraph.
```

```json
{
  "stdin": "- [x] Before `first  \r        second [x]( url )` after _outside_ [real](  target  ).\r\t \r      Next paragraph.\r",
  "reviewed_stdout": "- [x] Before `first  \r        second [x]( url )` after *outside* [real](target).\r\r      Next paragraph.\r",
  "current_stdout": "- [x] Before `first\r        second [x]( url )` after _outside_ [real](  target  ).\r\r      Next paragraph.\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-632

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-632) · [active transcript](../../cases/recovered_pr6_preserves_literals_in_recursive_footnotes_632.case) · [reviewed transcript](recovered_pr6_preserves_literals_in_recursive_footnotes_632.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
-    Before `first  
+    Before `first
       second\
     third [x]( url )` after *outside* [real](target).
 
```

```json
{
  "stdin": "[^note]: Before `first  \n      second\\\n    third [x]( url )` after _outside_ [real](  target  ).\n \t\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before `first  \n      second\\\n    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before `first\n      second\\\n    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-633

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-633)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
-    Before `first  
+    Before `first
       second\
     third [x]( url )` after *outside* [real](target).
 
```

```json
{
  "stdin": "[^note]:\n    Before `first  \n      second\\\n    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before `first  \n      second\\\n    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before `first\n      second\\\n    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-634

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-634) · [active transcript](../../cases/recovered_pr6_preserves_literals_in_recursive_footnotes_634.case) · [reviewed transcript](recovered_pr6_preserves_literals_in_recursive_footnotes_634.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 [^note]:
-    Before `first  
+    Before `first
       second\
-    third [x]( url )` after *outside* [real](target).
+    third [x]( url )` after _outside_ [real](  target  ).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]: Before `first  \n      second\\\n    third [x]( url )` after _outside_ [real](  target  ).\n \t\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before `first  \n      second\\\n    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before `first\n      second\\\n    third [x]( url )` after _outside_ [real](  target  ).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-635

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-635) · [active transcript](../../cases/recovered_pr6_preserves_literals_in_recursive_footnotes_635.case) · [reviewed transcript](recovered_pr6_preserves_literals_in_recursive_footnotes_635.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 [^note]:
     First paragraph.
 
-    Before `first  
+    Before `first
       second\
     third [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "[^note]: First paragraph.\n\n    Before `first  \n      second\\\n    third [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before `first  \n      second\\\n    third [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before `first\n      second\\\n    third [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-636

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-636)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 [^note]:
     First paragraph.
 
-    Before `first  
+    Before `first
       second\
     third [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "[^note]:\n    First paragraph.\n\n    Before `first  \n      second\\\n    third [x]( url )` after *outside* [real](target).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before `first  \n      second\\\n    third [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before `first\n      second\\\n    third [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-637

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-637) · [active transcript](../../cases/recovered_pr6_preserves_literals_in_recursive_footnotes_637.case) · [reviewed transcript](recovered_pr6_preserves_literals_in_recursive_footnotes_637.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 [^note]:
     First paragraph.
 
-    Before `first  
+    Before `first
       second\
-    third [x]( url )` after *outside* [real](target).
+    third [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "[^note]: First paragraph.\n\n    Before `first  \n      second\\\n    third [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before `first  \n      second\\\n    third [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before `first\n      second\\\n    third [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-638

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-638)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 [^note]:
-    Before `first  
-      second\
+    Before `first
+      second\
     third [x]( url )` after *outside* [real](target).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]: Before `first  \r\n      second\\\r\n    third [x]( url )` after _outside_ [real](  target  ).\n \t\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before `first  \r\n      second\\\r\n    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before `first\n      second\\\n    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-639

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-639)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 [^note]:
-    Before `first  
-      second\
+    Before `first
+      second\
     third [x]( url )` after *outside* [real](target).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]:\n    Before `first  \r\n      second\\\r\n    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before `first  \r\n      second\\\r\n    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before `first\n      second\\\n    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-640

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-640)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 [^note]:
-    Before `first  
-      second\
-    third [x]( url )` after *outside* [real](target).
+    Before `first
+      second\
+    third [x]( url )` after _outside_ [real](  target  ).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]: Before `first  \r\n      second\\\r\n    third [x]( url )` after _outside_ [real](  target  ).\n \t\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before `first  \r\n      second\\\r\n    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before `first\n      second\\\n    third [x]( url )` after _outside_ [real](  target  ).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-641

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-641)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 [^note]:
     First paragraph.
 
-    Before `first  
-      second\
+    Before `first
+      second\
     third [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "[^note]: First paragraph.\n\n    Before `first  \r\n      second\\\r\n    third [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before `first  \r\n      second\\\r\n    third [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before `first\n      second\\\n    third [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-642

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-642)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 [^note]:
     First paragraph.
 
-    Before `first  
-      second\
+    Before `first
+      second\
     third [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "[^note]:\n    First paragraph.\n\n    Before `first  \r\n      second\\\r\n    third [x]( url )` after *outside* [real](target).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before `first  \r\n      second\\\r\n    third [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before `first\n      second\\\n    third [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-643

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-643)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 [^note]:
     First paragraph.
 
-    Before `first  
-      second\
-    third [x]( url )` after *outside* [real](target).
+    Before `first
+      second\
+    third [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "[^note]: First paragraph.\n\n    Before `first  \r\n      second\\\r\n    third [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before `first  \r\n      second\\\r\n    third [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before `first\n      second\\\n    third [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-644

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-644)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 [^note]:
-    Before `first  -      second\+    Before `first
+      second\
     third [x]( url )` after *outside* [real](target).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]: Before `first  \r      second\\\r    third [x]( url )` after _outside_ [real](  target  ).\n \t\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before `first  \r      second\\\r    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before `first\n      second\\\n    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-645

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-645)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 [^note]:
-    Before `first  -      second\+    Before `first
+      second\
     third [x]( url )` after *outside* [real](target).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]:\n    Before `first  \r      second\\\r    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before `first  \r      second\\\r    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before `first\n      second\\\n    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-646

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-646)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 [^note]:
-    Before `first  -      second\-    third [x]( url )` after *outside* [real](target).
+    Before `first
+      second\
+    third [x]( url )` after _outside_ [real](  target  ).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]: Before `first  \r      second\\\r    third [x]( url )` after _outside_ [real](  target  ).\n \t\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before `first  \r      second\\\r    third [x]( url )` after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before `first\n      second\\\n    third [x]( url )` after _outside_ [real](  target  ).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-647

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-647)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 [^note]:
     First paragraph.
 
-    Before `first  -      second\+    Before `first
+      second\
     third [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "[^note]: First paragraph.\n\n    Before `first  \r      second\\\r    third [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before `first  \r      second\\\r    third [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before `first\n      second\\\n    third [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-648

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-648)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 [^note]:
     First paragraph.
 
-    Before `first  -      second\+    Before `first
+      second\
     third [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "[^note]:\n    First paragraph.\n\n    Before `first  \r      second\\\r    third [x]( url )` after *outside* [real](target).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before `first  \r      second\\\r    third [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before `first\n      second\\\n    third [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-649

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-649)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 [^note]:
     First paragraph.
 
-    Before `first  -      second\-    third [x]( url )` after *outside* [real](target).
+    Before `first
+      second\
+    third [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "[^note]: First paragraph.\n\n    Before `first  \r      second\\\r    third [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before `first  \r      second\\\r    third [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before `first\n      second\\\n    third [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-650

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-650) · [active transcript](../../cases/recovered_pr6_preserves_literals_in_recursive_footnotes_650.case) · [reviewed transcript](recovered_pr6_preserves_literals_in_recursive_footnotes_650.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
-    Before $first	
+    Before $first
       second [x]( url )$ after *outside* [real](target).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]: Before $first\t\n      second [x]( url )$ after _outside_ [real](  target  ).\n \t\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before $first\t\n      second [x]( url )$ after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before $first\n      second [x]( url )$ after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-651

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-651)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
-    Before $first	
+    Before $first
       second [x]( url )$ after *outside* [real](target).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]:\n    Before $first\t\n      second [x]( url )$ after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before $first\t\n      second [x]( url )$ after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before $first\n      second [x]( url )$ after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-652

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-652) · [active transcript](../../cases/recovered_pr6_preserves_literals_in_recursive_footnotes_652.case) · [reviewed transcript](recovered_pr6_preserves_literals_in_recursive_footnotes_652.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
     First paragraph.
 
-    Before $first	
+    Before $first
       second [x]( url )$ after *outside* [real](target).
```

```json
{
  "stdin": "[^note]: First paragraph.\n\n    Before $first\t\n      second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before $first\t\n      second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before $first\n      second [x]( url )$ after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-653

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-653)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
     First paragraph.
 
-    Before $first	
+    Before $first
       second [x]( url )$ after *outside* [real](target).
```

```json
{
  "stdin": "[^note]:\n    First paragraph.\n\n    Before $first\t\n      second [x]( url )$ after *outside* [real](target).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before $first\t\n      second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before $first\n      second [x]( url )$ after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-654

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-654)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
-    Before $first	
+    Before $first
       second [x]( url )$ after *outside* [real](target).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]: Before $first\t\r\n      second [x]( url )$ after _outside_ [real](  target  ).\n \t\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before $first\t\r\n      second [x]( url )$ after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before $first\n      second [x]( url )$ after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-655

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-655)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
-    Before $first	
+    Before $first
       second [x]( url )$ after *outside* [real](target).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]:\n    Before $first\t\r\n      second [x]( url )$ after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before $first\t\r\n      second [x]( url )$ after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before $first\n      second [x]( url )$ after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-656

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-656)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
     First paragraph.
 
-    Before $first	
+    Before $first
       second [x]( url )$ after *outside* [real](target).
```

```json
{
  "stdin": "[^note]: First paragraph.\n\n    Before $first\t\r\n      second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before $first\t\r\n      second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before $first\n      second [x]( url )$ after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-657

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-657)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
     First paragraph.
 
-    Before $first	
+    Before $first
       second [x]( url )$ after *outside* [real](target).
```

```json
{
  "stdin": "[^note]:\n    First paragraph.\n\n    Before $first\t\r\n      second [x]( url )$ after *outside* [real](target).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before $first\t\r\n      second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before $first\n      second [x]( url )$ after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-658

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-658)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
-    Before $first	+    Before $first
       second [x]( url )$ after *outside* [real](target).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]: Before $first\t\r      second [x]( url )$ after _outside_ [real](  target  ).\n \t\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before $first\t\r      second [x]( url )$ after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before $first\n      second [x]( url )$ after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-659

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-659)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
-    Before $first	+    Before $first
       second [x]( url )$ after *outside* [real](target).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]:\n    Before $first\t\r      second [x]( url )$ after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before $first\t\r      second [x]( url )$ after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before $first\n      second [x]( url )$ after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-660

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-660)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
     First paragraph.
 
-    Before $first	+    Before $first
       second [x]( url )$ after *outside* [real](target).
```

```json
{
  "stdin": "[^note]: First paragraph.\n\n    Before $first\t\r      second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before $first\t\r      second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before $first\n      second [x]( url )$ after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-661

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-661)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
     First paragraph.
 
-    Before $first	+    Before $first
       second [x]( url )$ after *outside* [real](target).
```

```json
{
  "stdin": "[^note]:\n    First paragraph.\n\n    Before $first\t\r      second [x]( url )$ after *outside* [real](target).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before $first\t\r      second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before $first\n      second [x]( url )$ after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-662

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-662) · [active transcript](../../cases/recovered_pr6_preserves_literals_in_recursive_footnotes_662.case) · [reviewed transcript](recovered_pr6_preserves_literals_in_recursive_footnotes_662.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
-    Before `first  
+    Before `first
       second {{< include file >}}` after *outside* [real](target).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]: Before `first  \n      second {{< include file >}}` after _outside_ [real](  target  ).\n \t\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before `first  \n      second {{< include file >}}` after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before `first\n      second {{< include file >}}` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-663

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-663)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
-    Before `first  
+    Before `first
       second {{< include file >}}` after *outside* [real](target).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]:\n    Before `first  \n      second {{< include file >}}` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before `first  \n      second {{< include file >}}` after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before `first\n      second {{< include file >}}` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-664

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-664) · [active transcript](../../cases/recovered_pr6_preserves_literals_in_recursive_footnotes_664.case) · [reviewed transcript](recovered_pr6_preserves_literals_in_recursive_footnotes_664.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
     First paragraph.
 
-    Before `first  
+    Before `first
       second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "[^note]: First paragraph.\n\n    Before `first  \n      second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before `first  \n      second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before `first\n      second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-665

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-665)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
     First paragraph.
 
-    Before `first  
+    Before `first
       second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "[^note]:\n    First paragraph.\n\n    Before `first  \n      second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before `first  \n      second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before `first\n      second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-666

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-666)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
-    Before `first  
+    Before `first
       second {{< include file >}}` after *outside* [real](target).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]: Before `first  \r\n      second {{< include file >}}` after _outside_ [real](  target  ).\n \t\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before `first  \r\n      second {{< include file >}}` after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before `first\n      second {{< include file >}}` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-667

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-667)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
-    Before `first  
+    Before `first
       second {{< include file >}}` after *outside* [real](target).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]:\n    Before `first  \r\n      second {{< include file >}}` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before `first  \r\n      second {{< include file >}}` after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before `first\n      second {{< include file >}}` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-668

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-668)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
     First paragraph.
 
-    Before `first  
+    Before `first
       second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "[^note]: First paragraph.\n\n    Before `first  \r\n      second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before `first  \r\n      second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before `first\n      second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-669

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-669)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
     First paragraph.
 
-    Before `first  
+    Before `first
       second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "[^note]:\n    First paragraph.\n\n    Before `first  \r\n      second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before `first  \r\n      second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before `first\n      second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-670

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-670)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
-    Before `first  +    Before `first
       second {{< include file >}}` after *outside* [real](target).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]: Before `first  \r      second {{< include file >}}` after _outside_ [real](  target  ).\n \t\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before `first  \r      second {{< include file >}}` after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before `first\n      second {{< include file >}}` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-671

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-671)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
-    Before `first  +    Before `first
       second {{< include file >}}` after *outside* [real](target).
 
     Next paragraph.
```

```json
{
  "stdin": "[^note]:\n    Before `first  \r      second {{< include file >}}` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stdout": "[^note]:\n    Before `first  \r      second {{< include file >}}` after *outside* [real](target).\n\n    Next paragraph.\n",
  "current_stdout": "[^note]:\n    Before `first\n      second {{< include file >}}` after *outside* [real](target).\n\n    Next paragraph.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-672

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-672)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
     First paragraph.
 
-    Before `first  +    Before `first
       second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "[^note]: First paragraph.\n\n    Before `first  \r      second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before `first  \r      second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before `first\n      second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-673

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-673)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 [^note]:
     First paragraph.
 
-    Before `first  +    Before `first
       second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "[^note]:\n    First paragraph.\n\n    Before `first  \r      second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stdout": "[^note]:\n    First paragraph.\n\n    Before `first  \r      second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "[^note]:\n    First paragraph.\n\n    Before `first\n      second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-674

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-674) · [active transcript](../../cases/recovered_pr6_normalizes_nested_output_once_674.case) · [reviewed transcript](recovered_pr6_normalizes_nested_output_once_674.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 ::: note
-Before `first  
+Before `first \
   second [x]( url )` after *outside*.
 
 Next paragraph.
```

```json
{
  "stdin": "::: note\nBefore `first  \n  second [x]( url )` after _outside_.\n \t\nNext paragraph.\t\n:::\n",
  "reviewed_stdout": "::: note\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n",
  "current_stdout": "::: note\nBefore `first \\\n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-675

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-675)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 ::: note
-Before `first  
+Before `first \
   second [x]( url )` after *outside*.
 
 Next paragraph.
```

```json
{
  "stdin": "::: note\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n",
  "reviewed_stdout": "::: note\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n",
  "current_stdout": "::: note\nBefore `first \\\n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-676

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-676) · [active transcript](../../cases/recovered_pr6_normalizes_nested_output_once_676.case) · [reviewed transcript](recovered_pr6_normalizes_nested_output_once_676.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 > ::: note
-> Before `first  
+> Before `first \
 >   second [x]( url )` after *outside*.
 >
 > Next paragraph.
```

```json
{
  "stdin": "> ::: note\n> Before `first  \n>   second [x]( url )` after _outside_.\n>   \n> Next paragraph.\t\n> :::\n",
  "reviewed_stdout": "> ::: note\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n",
  "current_stdout": "> ::: note\n> Before `first \\\n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-677

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-677)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 > ::: note
-> Before `first  
+> Before `first \
 >   second [x]( url )` after *outside*.
 >
 > Next paragraph.
```

```json
{
  "stdin": "> ::: note\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n",
  "reviewed_stdout": "> ::: note\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n",
  "current_stdout": "> ::: note\n> Before `first \\\n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-678

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-678) · [active transcript](../../cases/recovered_pr6_normalizes_nested_output_once_678.case) · [reviewed transcript](recovered_pr6_normalizes_nested_output_once_678.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 ::: note
-Before `first  
-  second [x]( url )` after *outside*.
+Before `first
+  second [x]( url )` after _outside_.
 
 Next paragraph.
 :::
```

```json
{
  "stdin": "::: note\nBefore `first  \n  second [x]( url )` after _outside_.\n \t\nNext paragraph.\t\n:::\n",
  "reviewed_stdout": "::: note\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n",
  "current_stdout": "::: note\nBefore `first\n  second [x]( url )` after _outside_.\n\nNext paragraph.\n:::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-679

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-679)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 ::: note
-Before `first  
+Before `first
   second [x]( url )` after *outside*.
 
 Next paragraph.
```

```json
{
  "stdin": "::: note\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n",
  "reviewed_stdout": "::: note\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n",
  "current_stdout": "::: note\nBefore `first\n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-680

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-680) · [active transcript](../../cases/recovered_pr6_normalizes_nested_output_once_680.case) · [reviewed transcript](recovered_pr6_normalizes_nested_output_once_680.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 > ::: note
-> Before `first  
->   second [x]( url )` after *outside*.
+> Before `first
+>   second [x]( url )` after _outside_.
 >
 > Next paragraph.
 > :::
```

```json
{
  "stdin": "> ::: note\n> Before `first  \n>   second [x]( url )` after _outside_.\n>   \n> Next paragraph.\t\n> :::\n",
  "reviewed_stdout": "> ::: note\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n",
  "current_stdout": "> ::: note\n> Before `first\n>   second [x]( url )` after _outside_.\n>\n> Next paragraph.\n> :::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-681

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-681)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 > ::: note
-> Before `first  
+> Before `first
 >   second [x]( url )` after *outside*.
 >
 > Next paragraph.
```

```json
{
  "stdin": "> ::: note\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n",
  "reviewed_stdout": "> ::: note\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n",
  "current_stdout": "> ::: note\n> Before `first\n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-682

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-682) · [active transcript](../../cases/recovered_pr6_normalizes_nested_output_once_682.case) · [reviewed transcript](recovered_pr6_normalizes_nested_output_once_682.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 :::: outer
 ::: inner
-Before `first  
+Before `first \
   second [x]( url )` after *outside*.
 
 Next paragraph.
```

```json
{
  "stdin": ":::: outer\n::: inner\nBefore `first  \n  second [x]( url )` after _outside_.\n \t\nNext paragraph.\t\n:::\n::::\n",
  "reviewed_stdout": ":::: outer\n::: inner\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n::::\n",
  "current_stdout": ":::: outer\n::: inner\nBefore `first \\\n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n::::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-683

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-683)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 :::: outer
 ::: inner
-Before `first  
+Before `first \
   second [x]( url )` after *outside*.
 
 Next paragraph.
```

```json
{
  "stdin": ":::: outer\n::: inner\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n::::\n",
  "reviewed_stdout": ":::: outer\n::: inner\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n::::\n",
  "current_stdout": ":::: outer\n::: inner\nBefore `first \\\n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n::::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-684

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-684) · [active transcript](../../cases/recovered_pr6_normalizes_nested_output_once_684.case) · [reviewed transcript](recovered_pr6_normalizes_nested_output_once_684.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 > :::: outer
 > ::: inner
-> Before `first  
+> Before `first \
 >   second [x]( url )` after *outside*.
 >
 > Next paragraph.
```

```json
{
  "stdin": "> :::: outer\n> ::: inner\n> Before `first  \n>   second [x]( url )` after _outside_.\n>   \n> Next paragraph.\t\n> :::\n> ::::\n",
  "reviewed_stdout": "> :::: outer\n> ::: inner\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n> ::::\n",
  "current_stdout": "> :::: outer\n> ::: inner\n> Before `first \\\n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n> ::::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-685

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-685)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 > :::: outer
 > ::: inner
-> Before `first  
+> Before `first \
 >   second [x]( url )` after *outside*.
 >
 > Next paragraph.
```

```json
{
  "stdin": "> :::: outer\n> ::: inner\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n> ::::\n",
  "reviewed_stdout": "> :::: outer\n> ::: inner\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n> ::::\n",
  "current_stdout": "> :::: outer\n> ::: inner\n> Before `first \\\n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n> ::::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-686

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-686) · [active transcript](../../cases/recovered_pr6_normalizes_nested_output_once_686.case) · [reviewed transcript](recovered_pr6_normalizes_nested_output_once_686.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,7 +1,7 @@
 :::: outer
 ::: inner
-Before `first  
-  second [x]( url )` after *outside*.
+Before `first
+  second [x]( url )` after _outside_.
 
 Next paragraph.
 :::
```

```json
{
  "stdin": ":::: outer\n::: inner\nBefore `first  \n  second [x]( url )` after _outside_.\n \t\nNext paragraph.\t\n:::\n::::\n",
  "reviewed_stdout": ":::: outer\n::: inner\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n::::\n",
  "current_stdout": ":::: outer\n::: inner\nBefore `first\n  second [x]( url )` after _outside_.\n\nNext paragraph.\n:::\n::::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-687

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-687)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 :::: outer
 ::: inner
-Before `first  
+Before `first
   second [x]( url )` after *outside*.
 
 Next paragraph.
```

```json
{
  "stdin": ":::: outer\n::: inner\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n::::\n",
  "reviewed_stdout": ":::: outer\n::: inner\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n::::\n",
  "current_stdout": ":::: outer\n::: inner\nBefore `first\n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n::::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-688

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-688) · [active transcript](../../cases/recovered_pr6_normalizes_nested_output_once_688.case) · [reviewed transcript](recovered_pr6_normalizes_nested_output_once_688.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,7 +1,7 @@
 > :::: outer
 > ::: inner
-> Before `first  
->   second [x]( url )` after *outside*.
+> Before `first
+>   second [x]( url )` after _outside_.
 >
 > Next paragraph.
 > :::
```

```json
{
  "stdin": "> :::: outer\n> ::: inner\n> Before `first  \n>   second [x]( url )` after _outside_.\n>   \n> Next paragraph.\t\n> :::\n> ::::\n",
  "reviewed_stdout": "> :::: outer\n> ::: inner\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n> ::::\n",
  "current_stdout": "> :::: outer\n> ::: inner\n> Before `first\n>   second [x]( url )` after _outside_.\n>\n> Next paragraph.\n> :::\n> ::::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-689

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-689)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 > :::: outer
 > ::: inner
-> Before `first  
+> Before `first
 >   second [x]( url )` after *outside*.
 >
 > Next paragraph.
```

```json
{
  "stdin": "> :::: outer\n> ::: inner\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n> ::::\n",
  "reviewed_stdout": "> :::: outer\n> ::: inner\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n> ::::\n",
  "current_stdout": "> :::: outer\n> ::: inner\n> Before `first\n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> :::\n> ::::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-690

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-690) · [active transcript](../../cases/recovered_pr6_normalizes_nested_output_once_690.case) · [reviewed transcript](recovered_pr6_normalizes_nested_output_once_690.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 ```markdown
-Before `first  
+Before `first \
   second [x]( url )` after *outside*.
 
 Next paragraph.
```

```json
{
  "stdin": "```markdown\nBefore `first  \n  second [x]( url )` after _outside_.\n \t\nNext paragraph.\t\n```\n",
  "reviewed_stdout": "```markdown\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n```\n",
  "current_stdout": "```markdown\nBefore `first \\\n  second [x]( url )` after *outside*.\n\nNext paragraph.\n```\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-691

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-691)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 ```markdown
-Before `first  
+Before `first \
   second [x]( url )` after *outside*.
 
 Next paragraph.
```

```json
{
  "stdin": "```markdown\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n```\n",
  "reviewed_stdout": "```markdown\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n```\n",
  "current_stdout": "```markdown\nBefore `first \\\n  second [x]( url )` after *outside*.\n\nNext paragraph.\n```\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-692

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-692) · [active transcript](../../cases/recovered_pr6_normalizes_nested_output_once_692.case) · [reviewed transcript](recovered_pr6_normalizes_nested_output_once_692.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 > ```markdown
-> Before `first  
+> Before `first \
 >   second [x]( url )` after *outside*.
 >
 > Next paragraph.
```

```json
{
  "stdin": "> ```markdown\n> Before `first  \n>   second [x]( url )` after _outside_.\n>   \n> Next paragraph.\t\n> ```\n",
  "reviewed_stdout": "> ```markdown\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> ```\n",
  "current_stdout": "> ```markdown\n> Before `first \\\n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> ```\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-693

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-693)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 > ```markdown
-> Before `first  
+> Before `first \
 >   second [x]( url )` after *outside*.
 >
 > Next paragraph.
```

```json
{
  "stdin": "> ```markdown\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> ```\n",
  "reviewed_stdout": "> ```markdown\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> ```\n",
  "current_stdout": "> ```markdown\n> Before `first \\\n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> ```\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-694

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-694) · [active transcript](../../cases/recovered_pr6_normalizes_nested_output_once_694.case) · [reviewed transcript](recovered_pr6_normalizes_nested_output_once_694.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 ```markdown
-Before `first  
-  second [x]( url )` after *outside*.
+Before `first
+  second [x]( url )` after _outside_.
 
 Next paragraph.
 ```
```

```json
{
  "stdin": "```markdown\nBefore `first  \n  second [x]( url )` after _outside_.\n \t\nNext paragraph.\t\n```\n",
  "reviewed_stdout": "```markdown\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n```\n",
  "current_stdout": "```markdown\nBefore `first\n  second [x]( url )` after _outside_.\n\nNext paragraph.\n```\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-695

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-695)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 ```markdown
-Before `first  
+Before `first
   second [x]( url )` after *outside*.
 
 Next paragraph.
```

```json
{
  "stdin": "```markdown\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n```\n",
  "reviewed_stdout": "```markdown\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n```\n",
  "current_stdout": "```markdown\nBefore `first\n  second [x]( url )` after *outside*.\n\nNext paragraph.\n```\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-696

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-696) · [active transcript](../../cases/recovered_pr6_normalizes_nested_output_once_696.case) · [reviewed transcript](recovered_pr6_normalizes_nested_output_once_696.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,6 @@
 > ```markdown
-> Before `first  
->   second [x]( url )` after *outside*.
+> Before `first
+>   second [x]( url )` after _outside_.
 >
 > Next paragraph.
 > ```
```

```json
{
  "stdin": "> ```markdown\n> Before `first  \n>   second [x]( url )` after _outside_.\n>   \n> Next paragraph.\t\n> ```\n",
  "reviewed_stdout": "> ```markdown\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> ```\n",
  "current_stdout": "> ```markdown\n> Before `first\n>   second [x]( url )` after _outside_.\n>\n> Next paragraph.\n> ```\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-697

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-697)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 80 --canonical
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 > ```markdown
-> Before `first  
+> Before `first
 >   second [x]( url )` after *outside*.
 >
 > Next paragraph.
```

```json
{
  "stdin": "> ```markdown\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> ```\n",
  "reviewed_stdout": "> ```markdown\n> Before `first  \n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> ```\n",
  "current_stdout": "> ```markdown\n> Before `first\n>   second [x]( url )` after *outside*.\n>\n> Next paragraph.\n> ```\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-708

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-708) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_708.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_708.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
-: Before `first  
-    second [x]( url )` after *outside* [real](target).
+: Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Term\n: Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n: Before `first  \n    second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "Term\n: Before `first\n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-709

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-709)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
-: Before `first  
+: Before `first
     second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n: Before `first  \n    second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stdout": "Term\n: Before `first  \n    second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "Term\n: Before `first\n    second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-710

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-710) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_710.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_710.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
 : Before $first\
-    second [x]( url )$ after *outside* [real](target).
+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Term\n: Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n: Before $first\\\n    second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "Term\n: Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-712

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-712) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_712.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_712.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,2 @@
 Term
-: Before `first	
-    second {{< include file >}}` after *outside* [real](target).
+: Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n: Before `first\t\n    second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n: Before `first\t\n    second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "Term\n: Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-713

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-713)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,2 @@
 Term
-: Before `first	
-    second {{< include file >}}` after *outside* [real](target).
+: Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n: Before `first\t\n    second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stdout": "Term\n: Before `first\t\n    second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "Term\n: Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-714

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-714) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_714.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_714.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
- : Before `first  
-    second [x]( url )` after *outside* [real](target).
+ : Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Term\n : Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n : Before `first  \n    second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "Term\n : Before `first\n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-715

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-715)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
- : Before `first  
+ : Before `first
     second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n : Before `first  \n    second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stdout": "Term\n : Before `first  \n    second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "Term\n : Before `first\n    second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-716

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-716) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_716.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_716.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
  : Before $first\
-    second [x]( url )$ after *outside* [real](target).
+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Term\n : Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n : Before $first\\\n    second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "Term\n : Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-718

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-718) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_718.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_718.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,2 @@
 Term
- : Before `first	
-    second {{< include file >}}` after *outside* [real](target).
+ : Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n : Before `first\t\n    second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n : Before `first\t\n    second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "Term\n : Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-719

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-719)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,2 @@
 Term
- : Before `first	
-    second {{< include file >}}` after *outside* [real](target).
+ : Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n : Before `first\t\n    second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stdout": "Term\n : Before `first\t\n    second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "Term\n : Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-720

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-720) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_720.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_720.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
-  : Before `first  
-    second [x]( url )` after *outside* [real](target).
+  : Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Term\n  : Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n  : Before `first  \n    second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "Term\n  : Before `first\n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-721

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-721)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
-  : Before `first  
+  : Before `first
     second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n  : Before `first  \n    second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stdout": "Term\n  : Before `first  \n    second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "Term\n  : Before `first\n    second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-722

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-722) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_722.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_722.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
   : Before $first\
-    second [x]( url )$ after *outside* [real](target).
+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Term\n  : Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n  : Before $first\\\n    second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "Term\n  : Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-724

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-724) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_724.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_724.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,2 @@
 Term
-  : Before `first	
-    second {{< include file >}}` after *outside* [real](target).
+  : Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n  : Before `first\t\n    second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n  : Before `first\t\n    second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "Term\n  : Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-725

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-725)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,2 @@
 Term
-  : Before `first	
-    second {{< include file >}}` after *outside* [real](target).
+  : Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n  : Before `first\t\n    second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stdout": "Term\n  : Before `first\t\n    second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "Term\n  : Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-726

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-726) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_726.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_726.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
-   : Before `first  
-     second [x]( url )` after *outside* [real](target).
+   : Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Term\n   : Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n   : Before `first  \n     second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "Term\n   : Before `first\n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-727

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-727)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
-   : Before `first  
+   : Before `first
      second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n   : Before `first  \n     second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stdout": "Term\n   : Before `first  \n     second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "Term\n   : Before `first\n     second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-728

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-728) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_728.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_728.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
    : Before $first\
-     second [x]( url )$ after *outside* [real](target).
+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Term\n   : Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n   : Before $first\\\n     second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "Term\n   : Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-730

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-730) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_730.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_730.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,2 @@
 Term
-   : Before `first	
-     second {{< include file >}}` after *outside* [real](target).
+   : Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n   : Before `first\t\n    second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n   : Before `first\t\n     second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "Term\n   : Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-731

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-731)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,2 @@
 Term
-   : Before `first	
-     second {{< include file >}}` after *outside* [real](target).
+   : Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n   : Before `first\t\n     second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stdout": "Term\n   : Before `first\t\n     second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "Term\n   : Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-732

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-732) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_732.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_732.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
-   ~ Before `first  
-     second [x]( url )` after *outside* [real](target).
+   ~ Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Term\n   ~ Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n   ~ Before `first  \n     second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "Term\n   ~ Before `first\n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-733

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-733)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
-   ~ Before `first  
+   ~ Before `first
      second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n   ~ Before `first  \n     second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stdout": "Term\n   ~ Before `first  \n     second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "Term\n   ~ Before `first\n     second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-734

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-734) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_734.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_734.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
    ~ Before $first\
-     second [x]( url )$ after *outside* [real](target).
+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Term\n   ~ Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n   ~ Before $first\\\n     second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "Term\n   ~ Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-736

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-736) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_736.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_736.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,2 @@
 Term
-   ~ Before `first	
-     second {{< include file >}}` after *outside* [real](target).
+   ~ Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n   ~ Before `first\t\n    second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n   ~ Before `first\t\n     second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "Term\n   ~ Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-737

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-737)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,2 @@
 Term
-   ~ Before `first	
-     second {{< include file >}}` after *outside* [real](target).
+   ~ Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n   ~ Before `first\t\n     second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stdout": "Term\n   ~ Before `first\t\n     second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "Term\n   ~ Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-738

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-738) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_738.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_738.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
-   : Before `first  
-       second [x]( url )` after *outside* [real](target).
+   : Before `first
+       second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Term\n   : Before `first  \n       second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n   : Before `first  \n       second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "Term\n   : Before `first\n       second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-739

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-739)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
-   : Before `first  
+   : Before `first
        second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n   : Before `first  \n       second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stdout": "Term\n   : Before `first  \n       second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "Term\n   : Before `first\n       second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-740

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-740) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_740.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_740.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 Term
    : Before $first\
-       second [x]( url )$ after *outside* [real](target).
+       second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Term\n   : Before $first\\\n       second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n   : Before $first\\\n       second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "Term\n   : Before $first\\\n       second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-742

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-742) · [active transcript](../../cases/recovered_pr6_accepts_definition_continuation_indentation_742.case) · [reviewed transcript](recovered_pr6_accepts_definition_continuation_indentation_742.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,2 @@
 Term
-   : Before `first	
-       second {{< include file >}}` after *outside* [real](target).
+   : Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n   : Before `first\t\n       second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Term\n   : Before `first\t\n       second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "Term\n   : Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-743

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-743)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,2 @@
 Term
-   : Before `first	
-       second {{< include file >}}` after *outside* [real](target).
+   : Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "Term\n   : Before `first\t\n       second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stdout": "Term\n   : Before `first\t\n       second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "Term\n   : Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-746

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-746) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_746.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_746.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1. Before `first  
-   second [x]( url )` after *outside* [real](target).
+1. Before `first
+ second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1. Before `first  \n second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "1. Before `first  \n   second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "1. Before `first\n second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-747

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-747)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1. Before `first  
+1. Before `first
    second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "1. Before `first  \n   second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stdout": "1. Before `first  \n   second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "1. Before `first\n   second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-748

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-748)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1. Before `first  
-   second [x]( url )` after *outside* [real](target).
+1. Before `first
+ second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1. Before `first  \r\n second [x]( url )` after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "1. Before `first  \r\n   second [x]( url )` after *outside* [real](target).\r\n",
  "current_stdout": "1. Before `first\r\n second [x]( url )` after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-749

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-749)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1. Before `first  
+1. Before `first
    second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "1. Before `first  \r\n   second [x]( url )` after *outside* [real](target).\r\n",
  "reviewed_stdout": "1. Before `first  \r\n   second [x]( url )` after *outside* [real](target).\r\n",
  "current_stdout": "1. Before `first\r\n   second [x]( url )` after *outside* [real](target).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-750

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-750)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1. Before `first  -   second [x]( url )` after *outside* [real](target).+1. Before `first+ second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1. Before `first  \r second [x]( url )` after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "1. Before `first  \r   second [x]( url )` after *outside* [real](target).\r",
  "current_stdout": "1. Before `first\r second [x]( url )` after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-751

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-751)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1. Before `first  +1. Before `first    second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "1. Before `first  \r   second [x]( url )` after *outside* [real](target).\r",
  "reviewed_stdout": "1. Before `first  \r   second [x]( url )` after *outside* [real](target).\r",
  "current_stdout": "1. Before `first\r   second [x]( url )` after *outside* [real](target).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-752

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-752) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_752.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_752.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 1. Before $first\
-   second [x]( url )$ after *outside* [real](target).
+ second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1. Before $first\\\n second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "1. Before $first\\\n   second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "1. Before $first\\\n second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-754

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-754)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 1. Before $first\
-   second [x]( url )$ after *outside* [real](target).
+ second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1. Before $first\\\r\n second [x]( url )$ after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "1. Before $first\\\r\n   second [x]( url )$ after *outside* [real](target).\r\n",
  "current_stdout": "1. Before $first\\\r\n second [x]( url )$ after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-756

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-756)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 1. Before $first\-   second [x]( url )$ after *outside* [real](target).+ second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1. Before $first\\\r second [x]( url )$ after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "1. Before $first\\\r   second [x]( url )$ after *outside* [real](target).\r",
  "current_stdout": "1. Before $first\\\r second [x]( url )$ after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-758

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-758) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_758.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_758.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1. Before `first	
-   second {{< include file >}}` after *outside* [real](target).
+1. Before `first
+ second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1. Before `first\t\n second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "1. Before `first\t\n   second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "1. Before `first\n second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-759

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-759)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-1. Before `first	
-   second {{< include file >}}` after *outside* [real](target).
+1. Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "1. Before `first\t\n   second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stdout": "1. Before `first\t\n   second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "1. Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-760

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-760)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1. Before `first	
-   second {{< include file >}}` after *outside* [real](target).
+1. Before `first
+ second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1. Before `first\t\r\n second {{< include file >}}` after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "1. Before `first\t\r\n   second {{< include file >}}` after *outside* [real](target).\r\n",
  "current_stdout": "1. Before `first\r\n second {{< include file >}}` after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-761

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-761)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-1. Before `first	
-   second {{< include file >}}` after *outside* [real](target).
+1. Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "1. Before `first\t\r\n   second {{< include file >}}` after *outside* [real](target).\r\n",
  "reviewed_stdout": "1. Before `first\t\r\n   second {{< include file >}}` after *outside* [real](target).\r\n",
  "current_stdout": "1. Before `first second {{< include file >}}` after *outside* [real](target).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-762

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-762)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1. Before `first	-   second {{< include file >}}` after *outside* [real](target).+1. Before `first+ second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1. Before `first\t\r second {{< include file >}}` after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "1. Before `first\t\r   second {{< include file >}}` after *outside* [real](target).\r",
  "current_stdout": "1. Before `first\r second {{< include file >}}` after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-763

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-763)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-1. Before `first	-   second {{< include file >}}` after *outside* [real](target).+1. Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "1. Before `first\t\r   second {{< include file >}}` after *outside* [real](target).\r",
  "reviewed_stdout": "1. Before `first\t\r   second {{< include file >}}` after *outside* [real](target).\r",
  "current_stdout": "1. Before `first second {{< include file >}}` after *outside* [real](target).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-764

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-764) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_764.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_764.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-10. Before `first  
-    second [x]( url )` after *outside* [real](target).
+10. Before `first
+  second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "10. Before `first  \n  second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "10. Before `first  \n    second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "10. Before `first\n  second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-765

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-765)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-10. Before `first  
+10. Before `first
     second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "10. Before `first  \n    second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stdout": "10. Before `first  \n    second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "10. Before `first\n    second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-766

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-766)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-10. Before `first  
-    second [x]( url )` after *outside* [real](target).
+10. Before `first
+  second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "10. Before `first  \r\n  second [x]( url )` after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "10. Before `first  \r\n    second [x]( url )` after *outside* [real](target).\r\n",
  "current_stdout": "10. Before `first\r\n  second [x]( url )` after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-767

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-767)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-10. Before `first  
+10. Before `first
     second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "10. Before `first  \r\n    second [x]( url )` after *outside* [real](target).\r\n",
  "reviewed_stdout": "10. Before `first  \r\n    second [x]( url )` after *outside* [real](target).\r\n",
  "current_stdout": "10. Before `first\r\n    second [x]( url )` after *outside* [real](target).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-768

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-768)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-10. Before `first  -    second [x]( url )` after *outside* [real](target).+10. Before `first+  second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "10. Before `first  \r  second [x]( url )` after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "10. Before `first  \r    second [x]( url )` after *outside* [real](target).\r",
  "current_stdout": "10. Before `first\r  second [x]( url )` after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-769

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-769)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-10. Before `first  +10. Before `first     second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "10. Before `first  \r    second [x]( url )` after *outside* [real](target).\r",
  "reviewed_stdout": "10. Before `first  \r    second [x]( url )` after *outside* [real](target).\r",
  "current_stdout": "10. Before `first\r    second [x]( url )` after *outside* [real](target).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-770

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-770) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_770.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_770.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 10. Before $first\
-    second [x]( url )$ after *outside* [real](target).
+  second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "10. Before $first\\\n  second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "10. Before $first\\\n    second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "10. Before $first\\\n  second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-772

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-772)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 10. Before $first\
-    second [x]( url )$ after *outside* [real](target).
+  second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "10. Before $first\\\r\n  second [x]( url )$ after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "10. Before $first\\\r\n    second [x]( url )$ after *outside* [real](target).\r\n",
  "current_stdout": "10. Before $first\\\r\n  second [x]( url )$ after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-774

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-774)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 10. Before $first\-    second [x]( url )$ after *outside* [real](target).+  second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "10. Before $first\\\r  second [x]( url )$ after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "10. Before $first\\\r    second [x]( url )$ after *outside* [real](target).\r",
  "current_stdout": "10. Before $first\\\r  second [x]( url )$ after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-776

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-776) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_776.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_776.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-10. Before `first	
-    second {{< include file >}}` after *outside* [real](target).
+10. Before `first
+  second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "10. Before `first\t\n  second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "10. Before `first\t\n    second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "10. Before `first\n  second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-777

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-777)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-10. Before `first	
-    second {{< include file >}}` after *outside* [real](target).
+10. Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "10. Before `first\t\n    second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stdout": "10. Before `first\t\n    second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "10. Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-778

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-778)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-10. Before `first	
-    second {{< include file >}}` after *outside* [real](target).
+10. Before `first
+  second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "10. Before `first\t\r\n  second {{< include file >}}` after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "10. Before `first\t\r\n    second {{< include file >}}` after *outside* [real](target).\r\n",
  "current_stdout": "10. Before `first\r\n  second {{< include file >}}` after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-779

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-779)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-10. Before `first	
-    second {{< include file >}}` after *outside* [real](target).
+10. Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "10. Before `first\t\r\n    second {{< include file >}}` after *outside* [real](target).\r\n",
  "reviewed_stdout": "10. Before `first\t\r\n    second {{< include file >}}` after *outside* [real](target).\r\n",
  "current_stdout": "10. Before `first second {{< include file >}}` after *outside* [real](target).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-780

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-780)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-10. Before `first	-    second {{< include file >}}` after *outside* [real](target).+10. Before `first+  second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "10. Before `first\t\r  second {{< include file >}}` after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "10. Before `first\t\r    second {{< include file >}}` after *outside* [real](target).\r",
  "current_stdout": "10. Before `first\r  second {{< include file >}}` after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-781

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-781)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-10. Before `first	-    second {{< include file >}}` after *outside* [real](target).+10. Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "10. Before `first\t\r    second {{< include file >}}` after *outside* [real](target).\r",
  "reviewed_stdout": "10. Before `first\t\r    second {{< include file >}}` after *outside* [real](target).\r",
  "current_stdout": "10. Before `first second {{< include file >}}` after *outside* [real](target).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-782

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-782) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_782.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_782.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first  
-     second [x]( url )` after *outside* [real](target).
+100. Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "100. Before `first  \n     second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "100. Before `first\n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-783

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-783)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first  
+100. Before `first
      second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "100. Before `first  \n     second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stdout": "100. Before `first  \n     second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "100. Before `first\n     second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-784

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-784)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first  
-     second [x]( url )` after *outside* [real](target).
+100. Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before `first  \r\n    second [x]( url )` after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "100. Before `first  \r\n     second [x]( url )` after *outside* [real](target).\r\n",
  "current_stdout": "100. Before `first\r\n    second [x]( url )` after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-785

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-785)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first  
+100. Before `first
      second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "100. Before `first  \r\n     second [x]( url )` after *outside* [real](target).\r\n",
  "reviewed_stdout": "100. Before `first  \r\n     second [x]( url )` after *outside* [real](target).\r\n",
  "current_stdout": "100. Before `first\r\n     second [x]( url )` after *outside* [real](target).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-786

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-786)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first  -     second [x]( url )` after *outside* [real](target).+100. Before `first+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before `first  \r    second [x]( url )` after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "100. Before `first  \r     second [x]( url )` after *outside* [real](target).\r",
  "current_stdout": "100. Before `first\r    second [x]( url )` after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-787

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-787)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first  +100. Before `first      second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "100. Before `first  \r     second [x]( url )` after *outside* [real](target).\r",
  "reviewed_stdout": "100. Before `first  \r     second [x]( url )` after *outside* [real](target).\r",
  "current_stdout": "100. Before `first\r     second [x]( url )` after *outside* [real](target).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-788

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-788) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_788.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_788.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 100. Before $first\
-     second [x]( url )$ after *outside* [real](target).
+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "100. Before $first\\\n     second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "100. Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-790

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-790)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 100. Before $first\
-     second [x]( url )$ after *outside* [real](target).
+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before $first\\\r\n    second [x]( url )$ after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "100. Before $first\\\r\n     second [x]( url )$ after *outside* [real](target).\r\n",
  "current_stdout": "100. Before $first\\\r\n    second [x]( url )$ after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-792

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-792)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 100. Before $first\-     second [x]( url )$ after *outside* [real](target).+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before $first\\\r    second [x]( url )$ after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "100. Before $first\\\r     second [x]( url )$ after *outside* [real](target).\r",
  "current_stdout": "100. Before $first\\\r    second [x]( url )$ after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-794

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-794) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_794.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_794.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first	
-     second {{< include file >}}` after *outside* [real](target).
+100. Before `first
+    second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before `first\t\n    second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "100. Before `first\t\n     second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "100. Before `first\n    second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-795

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-795)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-100. Before `first	
-     second {{< include file >}}` after *outside* [real](target).
+100. Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "100. Before `first\t\n     second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stdout": "100. Before `first\t\n     second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "100. Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-796

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-796)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first	
-     second {{< include file >}}` after *outside* [real](target).
+100. Before `first
+    second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before `first\t\r\n    second {{< include file >}}` after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "100. Before `first\t\r\n     second {{< include file >}}` after *outside* [real](target).\r\n",
  "current_stdout": "100. Before `first\r\n    second {{< include file >}}` after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-797

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-797)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-100. Before `first	
-     second {{< include file >}}` after *outside* [real](target).
+100. Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "100. Before `first\t\r\n     second {{< include file >}}` after *outside* [real](target).\r\n",
  "reviewed_stdout": "100. Before `first\t\r\n     second {{< include file >}}` after *outside* [real](target).\r\n",
  "current_stdout": "100. Before `first second {{< include file >}}` after *outside* [real](target).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-798

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-798)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first	-     second {{< include file >}}` after *outside* [real](target).+100. Before `first+    second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before `first\t\r    second {{< include file >}}` after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "100. Before `first\t\r     second {{< include file >}}` after *outside* [real](target).\r",
  "current_stdout": "100. Before `first\r    second {{< include file >}}` after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-799

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-799)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-100. Before `first	-     second {{< include file >}}` after *outside* [real](target).+100. Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "100. Before `first\t\r     second {{< include file >}}` after *outside* [real](target).\r",
  "reviewed_stdout": "100. Before `first\t\r     second {{< include file >}}` after *outside* [real](target).\r",
  "current_stdout": "100. Before `first second {{< include file >}}` after *outside* [real](target).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-800

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-800) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_800.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_800.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1000) Before `first  
-      second [x]( url )` after *outside* [real](target).
+1000) Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1000) Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "1000) Before `first  \n      second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "1000) Before `first\n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-801

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-801)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1000) Before `first  
+1000) Before `first
       second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "1000) Before `first  \n      second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stdout": "1000) Before `first  \n      second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "1000) Before `first\n      second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-802

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-802)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1000) Before `first  
-      second [x]( url )` after *outside* [real](target).
+1000) Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1000) Before `first  \r\n    second [x]( url )` after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "1000) Before `first  \r\n      second [x]( url )` after *outside* [real](target).\r\n",
  "current_stdout": "1000) Before `first\r\n    second [x]( url )` after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-803

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-803)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1000) Before `first  
+1000) Before `first
       second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "1000) Before `first  \r\n      second [x]( url )` after *outside* [real](target).\r\n",
  "reviewed_stdout": "1000) Before `first  \r\n      second [x]( url )` after *outside* [real](target).\r\n",
  "current_stdout": "1000) Before `first\r\n      second [x]( url )` after *outside* [real](target).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-804

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-804)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1000) Before `first  -      second [x]( url )` after *outside* [real](target).+1000) Before `first+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1000) Before `first  \r    second [x]( url )` after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "1000) Before `first  \r      second [x]( url )` after *outside* [real](target).\r",
  "current_stdout": "1000) Before `first\r    second [x]( url )` after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-805

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-805)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1000) Before `first  +1000) Before `first       second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "1000) Before `first  \r      second [x]( url )` after *outside* [real](target).\r",
  "reviewed_stdout": "1000) Before `first  \r      second [x]( url )` after *outside* [real](target).\r",
  "current_stdout": "1000) Before `first\r      second [x]( url )` after *outside* [real](target).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-806

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-806) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_806.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_806.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 1000) Before $first\
-      second [x]( url )$ after *outside* [real](target).
+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1000) Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "1000) Before $first\\\n      second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "1000) Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-808

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-808)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 1000) Before $first\
-      second [x]( url )$ after *outside* [real](target).
+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1000) Before $first\\\r\n    second [x]( url )$ after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "1000) Before $first\\\r\n      second [x]( url )$ after *outside* [real](target).\r\n",
  "current_stdout": "1000) Before $first\\\r\n    second [x]( url )$ after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-810

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-810)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 1000) Before $first\-      second [x]( url )$ after *outside* [real](target).+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1000) Before $first\\\r    second [x]( url )$ after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "1000) Before $first\\\r      second [x]( url )$ after *outside* [real](target).\r",
  "current_stdout": "1000) Before $first\\\r    second [x]( url )$ after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-812

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-812) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_812.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_812.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1000) Before `first	
-      second {{< include file >}}` after *outside* [real](target).
+1000) Before `first
+    second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1000) Before `first\t\n    second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "1000) Before `first\t\n      second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "1000) Before `first\n    second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-813

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-813)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-1000) Before `first	
-      second {{< include file >}}` after *outside* [real](target).
+1000) Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "1000) Before `first\t\n      second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stdout": "1000) Before `first\t\n      second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "1000) Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-814

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-814)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1000) Before `first	
-      second {{< include file >}}` after *outside* [real](target).
+1000) Before `first
+    second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1000) Before `first\t\r\n    second {{< include file >}}` after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "1000) Before `first\t\r\n      second {{< include file >}}` after *outside* [real](target).\r\n",
  "current_stdout": "1000) Before `first\r\n    second {{< include file >}}` after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-815

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-815)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-1000) Before `first	
-      second {{< include file >}}` after *outside* [real](target).
+1000) Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "1000) Before `first\t\r\n      second {{< include file >}}` after *outside* [real](target).\r\n",
  "reviewed_stdout": "1000) Before `first\t\r\n      second {{< include file >}}` after *outside* [real](target).\r\n",
  "current_stdout": "1000) Before `first second {{< include file >}}` after *outside* [real](target).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-816

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-816)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-1000) Before `first	-      second {{< include file >}}` after *outside* [real](target).+1000) Before `first+    second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "1000) Before `first\t\r    second {{< include file >}}` after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "1000) Before `first\t\r      second {{< include file >}}` after *outside* [real](target).\r",
  "current_stdout": "1000) Before `first\r    second {{< include file >}}` after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-817

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-817)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-1000) Before `first	-      second {{< include file >}}` after *outside* [real](target).+1000) Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "1000) Before `first\t\r      second {{< include file >}}` after *outside* [real](target).\r",
  "reviewed_stdout": "1000) Before `first\t\r      second {{< include file >}}` after *outside* [real](target).\r",
  "current_stdout": "1000) Before `first second {{< include file >}}` after *outside* [real](target).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-818

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-818) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_818.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_818.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- [x] Before `first  
-      second [x]( url )` after *outside* [real](target).
+- [x] Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- [x] Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "- [x] Before `first  \n      second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "- [x] Before `first\n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-819

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-819)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- [x] Before `first  
+- [x] Before `first
       second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "- [x] Before `first  \n      second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stdout": "- [x] Before `first  \n      second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "- [x] Before `first\n      second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-820

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-820)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- [x] Before `first  
-      second [x]( url )` after *outside* [real](target).
+- [x] Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- [x] Before `first  \r\n    second [x]( url )` after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "- [x] Before `first  \r\n      second [x]( url )` after *outside* [real](target).\r\n",
  "current_stdout": "- [x] Before `first\r\n    second [x]( url )` after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-821

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-821)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- [x] Before `first  
+- [x] Before `first
       second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "- [x] Before `first  \r\n      second [x]( url )` after *outside* [real](target).\r\n",
  "reviewed_stdout": "- [x] Before `first  \r\n      second [x]( url )` after *outside* [real](target).\r\n",
  "current_stdout": "- [x] Before `first\r\n      second [x]( url )` after *outside* [real](target).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-822

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-822)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- [x] Before `first  -      second [x]( url )` after *outside* [real](target).+- [x] Before `first+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- [x] Before `first  \r    second [x]( url )` after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "- [x] Before `first  \r      second [x]( url )` after *outside* [real](target).\r",
  "current_stdout": "- [x] Before `first\r    second [x]( url )` after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-823

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-823)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- [x] Before `first  +- [x] Before `first       second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "- [x] Before `first  \r      second [x]( url )` after *outside* [real](target).\r",
  "reviewed_stdout": "- [x] Before `first  \r      second [x]( url )` after *outside* [real](target).\r",
  "current_stdout": "- [x] Before `first\r      second [x]( url )` after *outside* [real](target).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-824

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-824) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_824.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_824.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 - [x] Before $first\
-      second [x]( url )$ after *outside* [real](target).
+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- [x] Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "- [x] Before $first\\\n      second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "- [x] Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-826

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-826)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 - [x] Before $first\
-      second [x]( url )$ after *outside* [real](target).
+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- [x] Before $first\\\r\n    second [x]( url )$ after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "- [x] Before $first\\\r\n      second [x]( url )$ after *outside* [real](target).\r\n",
  "current_stdout": "- [x] Before $first\\\r\n    second [x]( url )$ after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-828

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-828)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 - [x] Before $first\-      second [x]( url )$ after *outside* [real](target).+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- [x] Before $first\\\r    second [x]( url )$ after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "- [x] Before $first\\\r      second [x]( url )$ after *outside* [real](target).\r",
  "current_stdout": "- [x] Before $first\\\r    second [x]( url )$ after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-830

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-830) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_830.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_830.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- [x] Before `first	
-      second {{< include file >}}` after *outside* [real](target).
+- [x] Before `first
+    second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- [x] Before `first\t\n    second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "- [x] Before `first\t\n      second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "- [x] Before `first\n    second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-831

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-831)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-- [x] Before `first	
-      second {{< include file >}}` after *outside* [real](target).
+- [x] Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "- [x] Before `first\t\n      second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stdout": "- [x] Before `first\t\n      second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "- [x] Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-832

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-832)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- [x] Before `first	
-      second {{< include file >}}` after *outside* [real](target).
+- [x] Before `first
+    second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- [x] Before `first\t\r\n    second {{< include file >}}` after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "- [x] Before `first\t\r\n      second {{< include file >}}` after *outside* [real](target).\r\n",
  "current_stdout": "- [x] Before `first\r\n    second {{< include file >}}` after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-833

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-833)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-- [x] Before `first	
-      second {{< include file >}}` after *outside* [real](target).
+- [x] Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "- [x] Before `first\t\r\n      second {{< include file >}}` after *outside* [real](target).\r\n",
  "reviewed_stdout": "- [x] Before `first\t\r\n      second {{< include file >}}` after *outside* [real](target).\r\n",
  "current_stdout": "- [x] Before `first second {{< include file >}}` after *outside* [real](target).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-834

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-834)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-- [x] Before `first	-      second {{< include file >}}` after *outside* [real](target).+- [x] Before `first+    second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "- [x] Before `first\t\r    second {{< include file >}}` after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "- [x] Before `first\t\r      second {{< include file >}}` after *outside* [real](target).\r",
  "current_stdout": "- [x] Before `first\r    second {{< include file >}}` after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-835

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-835)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-- [x] Before `first	-      second {{< include file >}}` after *outside* [real](target).+- [x] Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "- [x] Before `first\t\r      second {{< include file >}}` after *outside* [real](target).\r",
  "reviewed_stdout": "- [x] Before `first\t\r      second {{< include file >}}` after *outside* [real](target).\r",
  "current_stdout": "- [x] Before `first second {{< include file >}}` after *outside* [real](target).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-836

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-836) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_836.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_836.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-  100. Before `first  
-       second [x]( url )` after *outside* [real](target).
+  100. Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "  100. Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "  100. Before `first  \n       second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "  100. Before `first\n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-837

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-837)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-  100. Before `first  
+  100. Before `first
        second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "  100. Before `first  \n       second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stdout": "  100. Before `first  \n       second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "  100. Before `first\n       second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-838

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-838)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-  100. Before `first  
-       second [x]( url )` after *outside* [real](target).
+  100. Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "  100. Before `first  \r\n    second [x]( url )` after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "  100. Before `first  \r\n       second [x]( url )` after *outside* [real](target).\r\n",
  "current_stdout": "  100. Before `first\r\n    second [x]( url )` after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-839

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-839)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-  100. Before `first  
+  100. Before `first
        second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "  100. Before `first  \r\n       second [x]( url )` after *outside* [real](target).\r\n",
  "reviewed_stdout": "  100. Before `first  \r\n       second [x]( url )` after *outside* [real](target).\r\n",
  "current_stdout": "  100. Before `first\r\n       second [x]( url )` after *outside* [real](target).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-840

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-840)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-  100. Before `first  -       second [x]( url )` after *outside* [real](target).+  100. Before `first+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "  100. Before `first  \r    second [x]( url )` after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "  100. Before `first  \r       second [x]( url )` after *outside* [real](target).\r",
  "current_stdout": "  100. Before `first\r    second [x]( url )` after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-841

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-841)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-  100. Before `first  +  100. Before `first        second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "  100. Before `first  \r       second [x]( url )` after *outside* [real](target).\r",
  "reviewed_stdout": "  100. Before `first  \r       second [x]( url )` after *outside* [real](target).\r",
  "current_stdout": "  100. Before `first\r       second [x]( url )` after *outside* [real](target).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-842

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-842) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_842.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_842.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
   100. Before $first\
-       second [x]( url )$ after *outside* [real](target).
+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "  100. Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "  100. Before $first\\\n       second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "  100. Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-844

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-844)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
   100. Before $first\
-       second [x]( url )$ after *outside* [real](target).
+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "  100. Before $first\\\r\n    second [x]( url )$ after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "  100. Before $first\\\r\n       second [x]( url )$ after *outside* [real](target).\r\n",
  "current_stdout": "  100. Before $first\\\r\n    second [x]( url )$ after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-846

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-846)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
   100. Before $first\-       second [x]( url )$ after *outside* [real](target).+    second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "  100. Before $first\\\r    second [x]( url )$ after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "  100. Before $first\\\r       second [x]( url )$ after *outside* [real](target).\r",
  "current_stdout": "  100. Before $first\\\r    second [x]( url )$ after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-848

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-848) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_848.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_848.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-  100. Before `first	
-       second {{< include file >}}` after *outside* [real](target).
+  100. Before `first
+    second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "  100. Before `first\t\n    second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "  100. Before `first\t\n       second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "  100. Before `first\n    second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-849

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-849)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-  100. Before `first	
-       second {{< include file >}}` after *outside* [real](target).
+  100. Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "  100. Before `first\t\n       second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stdout": "  100. Before `first\t\n       second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "  100. Before `first second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-850

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-850)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-  100. Before `first	
-       second {{< include file >}}` after *outside* [real](target).
+  100. Before `first
+    second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "  100. Before `first\t\r\n    second {{< include file >}}` after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "  100. Before `first\t\r\n       second {{< include file >}}` after *outside* [real](target).\r\n",
  "current_stdout": "  100. Before `first\r\n    second {{< include file >}}` after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-851

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-851)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-  100. Before `first	
-       second {{< include file >}}` after *outside* [real](target).
+  100. Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "  100. Before `first\t\r\n       second {{< include file >}}` after *outside* [real](target).\r\n",
  "reviewed_stdout": "  100. Before `first\t\r\n       second {{< include file >}}` after *outside* [real](target).\r\n",
  "current_stdout": "  100. Before `first second {{< include file >}}` after *outside* [real](target).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-852

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-852)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-  100. Before `first	-       second {{< include file >}}` after *outside* [real](target).+  100. Before `first+    second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "  100. Before `first\t\r    second {{< include file >}}` after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "  100. Before `first\t\r       second {{< include file >}}` after *outside* [real](target).\r",
  "current_stdout": "  100. Before `first\r    second {{< include file >}}` after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-853

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-853)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1 @@
-  100. Before `first	-       second {{< include file >}}` after *outside* [real](target).+  100. Before `first second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "  100. Before `first\t\r       second {{< include file >}}` after *outside* [real](target).\r",
  "reviewed_stdout": "  100. Before `first\t\r       second {{< include file >}}` after *outside* [real](target).\r",
  "current_stdout": "  100. Before `first second {{< include file >}}` after *outside* [real](target).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-854

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-854) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_854.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_854.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first  
-         second [x]( url )` after *outside* [real](target).
+100. Before `first
+         second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before `first  \n         second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "100. Before `first  \n         second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "100. Before `first\n         second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-855

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-855)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first  
+100. Before `first
          second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "100. Before `first  \n         second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stdout": "100. Before `first  \n         second [x]( url )` after *outside* [real](target).\n",
  "current_stdout": "100. Before `first\n         second [x]( url )` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-856

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-856)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first  
-         second [x]( url )` after *outside* [real](target).
+100. Before `first
+         second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before `first  \r\n         second [x]( url )` after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "100. Before `first  \r\n         second [x]( url )` after *outside* [real](target).\r\n",
  "current_stdout": "100. Before `first\r\n         second [x]( url )` after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-857

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-857)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first  
+100. Before `first
          second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "100. Before `first  \r\n         second [x]( url )` after *outside* [real](target).\r\n",
  "reviewed_stdout": "100. Before `first  \r\n         second [x]( url )` after *outside* [real](target).\r\n",
  "current_stdout": "100. Before `first\r\n         second [x]( url )` after *outside* [real](target).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-858

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-858)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first  -         second [x]( url )` after *outside* [real](target).+100. Before `first+         second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before `first  \r         second [x]( url )` after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "100. Before `first  \r         second [x]( url )` after *outside* [real](target).\r",
  "current_stdout": "100. Before `first\r         second [x]( url )` after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-859

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-859)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first  +100. Before `first          second [x]( url )` after *outside* [real](target).
```

```json
{
  "stdin": "100. Before `first  \r         second [x]( url )` after *outside* [real](target).\r",
  "reviewed_stdout": "100. Before `first  \r         second [x]( url )` after *outside* [real](target).\r",
  "current_stdout": "100. Before `first\r         second [x]( url )` after *outside* [real](target).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-860

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-860) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_860.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_860.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 100. Before $first\
-         second [x]( url )$ after *outside* [real](target).
+         second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before $first\\\n         second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "100. Before $first\\\n         second [x]( url )$ after *outside* [real](target).\n",
  "current_stdout": "100. Before $first\\\n         second [x]( url )$ after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-862

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-862)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 100. Before $first\
-         second [x]( url )$ after *outside* [real](target).
+         second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before $first\\\r\n         second [x]( url )$ after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "100. Before $first\\\r\n         second [x]( url )$ after *outside* [real](target).\r\n",
  "current_stdout": "100. Before $first\\\r\n         second [x]( url )$ after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-864

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-864)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
 100. Before $first\-         second [x]( url )$ after *outside* [real](target).+         second [x]( url )$ after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before $first\\\r         second [x]( url )$ after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "100. Before $first\\\r         second [x]( url )$ after *outside* [real](target).\r",
  "current_stdout": "100. Before $first\\\r         second [x]( url )$ after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-866

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-866) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_866.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_866.case)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first	
-         second {{< include file >}}` after *outside* [real](target).
+100. Before `first
+         second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before `first\t\n         second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "100. Before `first\t\n         second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "100. Before `first\n         second {{< include file >}}` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-867

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-867)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first	
+100. Before `first
          second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "100. Before `first\t\n         second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stdout": "100. Before `first\t\n         second {{< include file >}}` after *outside* [real](target).\n",
  "current_stdout": "100. Before `first\n         second {{< include file >}}` after *outside* [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-868

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-868)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first	
-         second {{< include file >}}` after *outside* [real](target).
+100. Before `first
+         second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before `first\t\r\n         second {{< include file >}}` after _outside_ [real](  target  ).\r\n",
  "reviewed_stdout": "100. Before `first\t\r\n         second {{< include file >}}` after *outside* [real](target).\r\n",
  "current_stdout": "100. Before `first\r\n         second {{< include file >}}` after _outside_ [real](  target  ).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-869

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-869)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first	
+100. Before `first
          second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "100. Before `first\t\r\n         second {{< include file >}}` after *outside* [real](target).\r\n",
  "reviewed_stdout": "100. Before `first\t\r\n         second {{< include file >}}` after *outside* [real](target).\r\n",
  "current_stdout": "100. Before `first\r\n         second {{< include file >}}` after *outside* [real](target).\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-870

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-870)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first	-         second {{< include file >}}` after *outside* [real](target).+100. Before `first+         second {{< include file >}}` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before `first\t\r         second {{< include file >}}` after _outside_ [real](  target  ).\r",
  "reviewed_stdout": "100. Before `first\t\r         second {{< include file >}}` after *outside* [real](target).\r",
  "current_stdout": "100. Before `first\r         second {{< include file >}}` after _outside_ [real](  target  ).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-871

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-871)

```text
format --stdin-file-path input.md --wrap none --canonical
format --stdin-file-path input.md --wrap paragraph --canonical
format --stdin-file-path input.md --wrap sentence --canonical
format --stdin-file-path input.md --wrap 120 --canonical
format --stdin-file-path input.md --wrap sentence:120 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,2 @@
-100. Before `first	+100. Before `first          second {{< include file >}}` after *outside* [real](target).
```

```json
{
  "stdin": "100. Before `first\t\r         second {{< include file >}}` after *outside* [real](target).\r",
  "reviewed_stdout": "100. Before `first\t\r         second {{< include file >}}` after *outside* [real](target).\r",
  "current_stdout": "100. Before `first\r         second {{< include file >}}` after *outside* [real](target).\r",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-872

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-872) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_872.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_872.case)

```text
format --stdin-file-path input.md --wrap sentence --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
 ::: note
-100. Before `first  
-     second [x]( url )` after *outside* [real](target).
+100. Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
 :::
```

```json
{
  "stdin": "::: note\n100. Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n:::\n",
  "reviewed_stdout": "::: note\n100. Before `first  \n     second [x]( url )` after *outside* [real](target).\n:::\n",
  "current_stdout": "::: note\n100. Before `first\n    second [x]( url )` after _outside_ [real](  target  ).\n:::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-873

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-873)

```text
format --stdin-file-path input.md --wrap sentence --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
 ::: note
-100. Before `first  
+100. Before `first
      second [x]( url )` after *outside* [real](target).
 :::
```

```json
{
  "stdin": "::: note\n100. Before `first  \n     second [x]( url )` after *outside* [real](target).\n:::\n",
  "reviewed_stdout": "::: note\n100. Before `first  \n     second [x]( url )` after *outside* [real](target).\n:::\n",
  "current_stdout": "::: note\n100. Before `first\n     second [x]( url )` after *outside* [real](target).\n:::\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-874

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-874) · [active transcript](../../cases/recovered_pr6_accepts_list_literal_continuation_indentation_874.case) · [reviewed transcript](recovered_pr6_accepts_list_literal_continuation_indentation_874.case)

```text
format --stdin-file-path input.md --wrap 24 --canonical
format --stdin-file-path input.md --wrap sentence:24 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,2 @@
-100. Before
-     `first  
-     second [x]( url )`
-     after *outside*
-     [real](target).
+100. Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "100. Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "100. Before\n     `first  \n     second [x]( url )`\n     after *outside*\n     [real](target).\n",
  "current_stdout": "100. Before `first\n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-875

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-875)

```text
format --stdin-file-path input.md --wrap 24 --canonical
format --stdin-file-path input.md --wrap sentence:24 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 100. Before
-     `first  
+     `first
      second [x]( url )`
      after *outside*
      [real](target).
```

```json
{
  "stdin": "100. Before\n     `first  \n     second [x]( url )`\n     after *outside*\n     [real](target).\n",
  "reviewed_stdout": "100. Before\n     `first  \n     second [x]( url )`\n     after *outside*\n     [real](target).\n",
  "current_stdout": "100. Before\n     `first\n     second [x]( url )`\n     after *outside*\n     [real](target).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-879

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-879) · [active transcript](../../cases/recovered_pr6_markdown_inline_preservation_canonical.case) · [reviewed transcript](recovered_pr6_markdown_inline_preservation_canonical.case)

```text
format --stdin-file-path input.md --wrap none --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,19 +1,19 @@
-# *Before `_a_ [x]( url )` after*
+# *Before `*a_ [x]( url )` after_
 
-# *Before $x + _a_ [x]( url )$ after*
+# *Before $x + _a* [x]( url )$ after_
 
 Before
-$a \$ _literal_ [x](  url  )$ after *outside*.
+$a \$ *literal* [x](  url  )$ after *outside*.
 
 - Before `  [x]( url ) \\ _literal_  ` after *outside*.
 
-> Before $a \$ _literal_ [x](  url  )$ after *outside*.
+> Before $a \$ *literal* [x](  url  )$ after *outside*.
 
 Before *with `[x]( url )` inside* after.
 
 Before ~~with `[x]( url )` and [real](target)~~ after.
 
-Before <kbd>`[x]( url )` and [real](target)</kbd> after.
+Before <kbd>`[x]( url )` and [real](  target  )</kbd> after.
 
 # *before \` literal* after `code`
 
@@ -39,8 +39,8 @@
 first\ second _outside_ [real](  url  ).
 
 Term
-   : Before `first  
-     second [x]( url )` after *outside* [real](target).
+   : Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
 
-- [x] Before `first	
-      second {{< include file >}}` after *outside*.
+- [x] Before `first
+    second {{< include file >}}` after _outside_.
```

```json
{
  "stdin": "# _Before `_a_ [x]( url )` after_\n\n# _Before $x + _a_ [x]( url )$ after_\n\nBefore\n$a \\$ _literal_ [x](  url  )$ after _outside_.\n\n- Before\n  `  [x]( url ) \\\\ _literal_  ` after _outside_.\n\n> Before\n> $a \\$ _literal_ [x](  url  )$ after _outside_.\n\nBefore _with `[x]( url )` inside_ after.\n\nBefore ~~with `[x]( url )` and [real](  target  )~~ after.\n\nBefore <kbd>`[x]( url )` and [real](  target  )</kbd> after.\n\n# _before \\` literal_ after `code`\n\n_before \\[ literal_ after ](target).\n\n- _before \\` literal_ after `code`\n\n> _before \\[ literal_ after ] text.\n\nfirst\fsecond `a\fb` $c\fd$\n\nfirst\f  \nsecond\n\nfirst\\   second `[x](  url  ) \\   ` _outside_ [real](  url  ).\n\n- first\\\tsecond `[x](  url  ) \\\t` _outside_ [real](  url  ).\n\nfirst\\ second _outside_ [real](  url  ).\n\nTerm\n   : Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n\n- [x] Before `first\t\n    second {{< include file >}}` after _outside_.\n",
  "reviewed_stdout": "# *Before `_a_ [x]( url )` after*\n\n# *Before $x + _a_ [x]( url )$ after*\n\nBefore\n$a \\$ _literal_ [x](  url  )$ after *outside*.\n\n- Before `  [x]( url ) \\\\ _literal_  ` after *outside*.\n\n> Before $a \\$ _literal_ [x](  url  )$ after *outside*.\n\nBefore *with `[x]( url )` inside* after.\n\nBefore ~~with `[x]( url )` and [real](target)~~ after.\n\nBefore <kbd>`[x]( url )` and [real](target)</kbd> after.\n\n# *before \\` literal* after `code`\n\n*before \\[ literal* after ](target).\n\n- *before \\` literal* after `code`\n\n> *before \\[ literal* after ] text.\n\nfirst\fsecond `a\fb` $c\fd$\n\nfirst\f \\\nsecond\n\nfirst\\   second `[x](  url  ) \\   ` *outside* [real](url).\n\n- first\\ second `[x](  url  ) \\\t` *outside* [real](url).\n\nfirst\\ second _outside_ [real](  url  ).\n\nTerm\n   : Before `first  \n     second [x]( url )` after *outside* [real](target).\n\n- [x] Before `first\t\n      second {{< include file >}}` after *outside*.\n",
  "current_stdout": "# *Before `*a_ [x]( url )` after_\n\n# *Before $x + _a* [x]( url )$ after_\n\nBefore\n$a \\$ *literal* [x](  url  )$ after *outside*.\n\n- Before `  [x]( url ) \\\\ _literal_  ` after *outside*.\n\n> Before $a \\$ *literal* [x](  url  )$ after *outside*.\n\nBefore *with `[x]( url )` inside* after.\n\nBefore ~~with `[x]( url )` and [real](target)~~ after.\n\nBefore <kbd>`[x]( url )` and [real](  target  )</kbd> after.\n\n# *before \\` literal* after `code`\n\n*before \\[ literal* after ](target).\n\n- *before \\` literal* after `code`\n\n> *before \\[ literal* after ] text.\n\nfirst\fsecond `a\fb` $c\fd$\n\nfirst\f \\\nsecond\n\nfirst\\   second `[x](  url  ) \\   ` *outside* [real](url).\n\n- first\\ second `[x](  url  ) \\\t` *outside* [real](url).\n\nfirst\\ second _outside_ [real](  url  ).\n\nTerm\n   : Before `first\n    second [x]( url )` after _outside_ [real](  target  ).\n\n- [x] Before `first\n    second {{< include file >}}` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-880

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-880) · [active transcript](../../cases/recovered_pr6_markdown_inline_preservation_column.case) · [reviewed transcript](recovered_pr6_markdown_inline_preservation_column.case)

```text
format --stdin-file-path input.md --wrap 24 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -3,7 +3,7 @@
 after [real](target).
 
 Before
-$a \$ _literal_ [x](  long-link-target  )$
+$a \$ *literal* [x](  long-link-target  )$
 after *outside*.
 
 - Before
@@ -42,8 +42,5 @@
      *outside*
      [real](target).
 
-100. Before
-     `first  
-     second [x]( url )`
-     after *outside*
-     [real](target).
+100. Before `first
+    second [x]( url )` after _outside_ [real](  target  ).
```

```json
{
  "stdin": "Before ``[x](  long-link-target  ) ` _literal_`` after [real](  target  ).\n\nBefore $a \\$ _literal_ [x](  long-link-target  )$ after _outside_.\n\n- Before `[x](  long-link-target  )` after [real](  target  ).\n\n> Before `[x](  long-link-target  ) {{< include file >}}` after _outside_.\n\nfirst\fsecond `a\fb` $c\fd$\n\n- first\fsecond `a\fb` $c\fd$\n\n> first\fsecond `a\fb` $c\fd$\n\nfirst\\   second `[x](  url  ) \\   ` after _outside_.\n\n- First paragraph.\n  \t\n  Next paragraph [real](  target  ).\n\nTerm\n   : Before\n    after _outside_ [real](  target  ).\n\n100. Before `first  \n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stdout": "Before\n``[x](  long-link-target  ) ` _literal_``\nafter [real](target).\n\nBefore\n$a \\$ _literal_ [x](  long-link-target  )$\nafter *outside*.\n\n- Before\n  `[x](  long-link-target  )`\n  after [real](target).\n\n> Before\n> `[x](  long-link-target  ) {{< include file >}}`\n> after *outside*.\n\nfirst second `a\fb` $c\fd$\n\n- first second `a\fb`\n  $c\fd$\n\n> first second `a\fb`\n> $c\fd$\n\nfirst\\ second\n`[x](  url  ) \\   `\nafter *outside*.\n\n- First paragraph.\n\n  Next paragraph\n  [real](target).\n\nTerm\n   : Before after\n     *outside*\n     [real](target).\n\n100. Before\n     `first  \n     second [x]( url )`\n     after *outside*\n     [real](target).\n",
  "current_stdout": "Before\n``[x](  long-link-target  ) ` _literal_``\nafter [real](target).\n\nBefore\n$a \\$ *literal* [x](  long-link-target  )$\nafter *outside*.\n\n- Before\n  `[x](  long-link-target  )`\n  after [real](target).\n\n> Before\n> `[x](  long-link-target  ) {{< include file >}}`\n> after *outside*.\n\nfirst second `a\fb` $c\fd$\n\n- first second `a\fb`\n  $c\fd$\n\n> first second `a\fb`\n> $c\fd$\n\nfirst\\ second\n`[x](  url  ) \\   `\nafter *outside*.\n\n- First paragraph.\n\n  Next paragraph\n  [real](target).\n\nTerm\n   : Before after\n     *outside*\n     [real](target).\n\n100. Before `first\n    second [x]( url )` after _outside_ [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-881

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-881) · [active transcript](../../cases/recovered_pr6_markdown_inline_preservation_multiline.case) · [reviewed transcript](recovered_pr6_markdown_inline_preservation_multiline.case)

```text
format --stdin-file-path input.md --wrap sentence:80 --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,43 +1,37 @@
-Before `first  
-  second [x]( url )` after *outside*.
-Next sentence.
+Before `first
+  second [x]( url )` after _outside_. Next sentence.
 
-- Before `first  
-    second [x]( url )` after *outside*.
-  Next sentence.
+- Before `first
+    second [x]( url )` after _outside_. Next sentence.
 
-> Before $first  
->   second \$ [x]( url )$ after *outside*.
-> Next sentence.
+> Before $first
+>   second \$ [x]( url )$ after _outside_. Next sentence.
 
 > > Before `first\
-> >   second [x]( url )` after *outside*.
-> > Next sentence.
+> >   second [x]( url )` after _outside_. Next sentence.
 
 Term
-: Before `first  
-      second [x]( url )` after *outside*.
+: Before `first
+      second [x]( url )` after _outside_.
 
-[^note]: Before `first  
-    second [x]( url )` after *outside*.
+[^note]: Before `first
+    second [x]( url )` after _outside_.
 
-- Béfore <kbd>`first  
-    second [x]( url )`</kbd> after *outside*
-  [real](target).
+- Béfore <kbd>`first
+    second [x]( url )`</kbd> after _outside_ [real](  target  ).
 
 - Before <kbd><span>$first\
-    second [x]( url )$</span></kbd> after *outside*
-  [real](target).
+    second [x]( url )$</span></kbd> after _outside_ [real](  target  ).
 
 ::: note
-Before `first  
-  second [x]( url )` after *outside*.
+Before `first
+  second [x]( url )` after _outside_.
 
 Next paragraph.
 :::
 
 ```markdown
-Before $first	
+Before $first
   second [x]( url )$ after *outside*.
 
 Next paragraph.
@@ -46,14 +40,12 @@
 [^recursive]:
     First paragraph.
 
-    Before `first  
+    Before `first
       second [x]( url )` after *outside*.
 
 Term
-   : Before `first  
-     second [x]( url )` after *outside*.
-     Next sentence.
+   : Before `first
+    second [x]( url )` after _outside_. Next sentence.
 
 100. Before $first\
-     second [x]( url )$ after *outside*.
-     Next sentence.
+    second [x]( url )$ after _outside_. Next sentence.
```

```json
{
  "stdin": "Before `first  \n  second [x]( url )` after _outside_. Next sentence.\n\n- Before `first  \n    second [x]( url )` after _outside_. Next sentence.\n\n> Before $first  \n>   second \\$ [x]( url )$ after _outside_. Next sentence.\n\n> > Before `first\\\n> >   second [x]( url )` after _outside_. Next sentence.\n\nTerm\n: Before `first  \n      second [x]( url )` after _outside_.\n\n[^note]: Before `first  \n    second [x]( url )` after _outside_.\n\n- Béfore <kbd>`first  \n    second [x]( url )`</kbd> after _outside_ [real](  target  ).\n\n- Before <kbd><span>$first\\\n    second [x]( url )$</span></kbd> after _outside_ [real](  target  ).\n\n::: note\nBefore `first  \n  second [x]( url )` after _outside_.\n \t\nNext paragraph.\t\n:::\n\n```markdown\nBefore $first\t\n  second [x]( url )$ after _outside_.\n \t\nNext paragraph.\t\n```\n\n[^recursive]: First paragraph.\n\n    Before `first  \n      second [x]( url )` after _outside_.\n\nTerm\n   : Before `first  \n    second [x]( url )` after _outside_. Next sentence.\n\n100. Before $first\\\n    second [x]( url )$ after _outside_. Next sentence.\n",
  "reviewed_stdout": "Before `first  \n  second [x]( url )` after *outside*.\nNext sentence.\n\n- Before `first  \n    second [x]( url )` after *outside*.\n  Next sentence.\n\n> Before $first  \n>   second \\$ [x]( url )$ after *outside*.\n> Next sentence.\n\n> > Before `first\\\n> >   second [x]( url )` after *outside*.\n> > Next sentence.\n\nTerm\n: Before `first  \n      second [x]( url )` after *outside*.\n\n[^note]: Before `first  \n    second [x]( url )` after *outside*.\n\n- Béfore <kbd>`first  \n    second [x]( url )`</kbd> after *outside*\n  [real](target).\n\n- Before <kbd><span>$first\\\n    second [x]( url )$</span></kbd> after *outside*\n  [real](target).\n\n::: note\nBefore `first  \n  second [x]( url )` after *outside*.\n\nNext paragraph.\n:::\n\n```markdown\nBefore $first\t\n  second [x]( url )$ after *outside*.\n\nNext paragraph.\n```\n\n[^recursive]:\n    First paragraph.\n\n    Before `first  \n      second [x]( url )` after *outside*.\n\nTerm\n   : Before `first  \n     second [x]( url )` after *outside*.\n     Next sentence.\n\n100. Before $first\\\n     second [x]( url )$ after *outside*.\n     Next sentence.\n",
  "current_stdout": "Before `first\n  second [x]( url )` after _outside_. Next sentence.\n\n- Before `first\n    second [x]( url )` after _outside_. Next sentence.\n\n> Before $first\n>   second \\$ [x]( url )$ after _outside_. Next sentence.\n\n> > Before `first\\\n> >   second [x]( url )` after _outside_. Next sentence.\n\nTerm\n: Before `first\n      second [x]( url )` after _outside_.\n\n[^note]: Before `first\n    second [x]( url )` after _outside_.\n\n- Béfore <kbd>`first\n    second [x]( url )`</kbd> after _outside_ [real](  target  ).\n\n- Before <kbd><span>$first\\\n    second [x]( url )$</span></kbd> after _outside_ [real](  target  ).\n\n::: note\nBefore `first\n  second [x]( url )` after _outside_.\n\nNext paragraph.\n:::\n\n```markdown\nBefore $first\n  second [x]( url )$ after *outside*.\n\nNext paragraph.\n```\n\n[^recursive]:\n    First paragraph.\n\n    Before `first\n      second [x]( url )` after *outside*.\n\nTerm\n   : Before `first\n    second [x]( url )` after _outside_. Next sentence.\n\n100. Before $first\\\n    second [x]( url )$ after _outside_. Next sentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-882

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-882) · [active transcript](../../cases/recovered_pr6_markdown_inline_preservation_paragraph.case) · [reviewed transcript](recovered_pr6_markdown_inline_preservation_paragraph.case)

```text
format --stdin-file-path input.md --wrap paragraph --canonical
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,16 +1,18 @@
-Before `first  
-  second {{< include file >}}` after *outside*.
+Before
+`first
+  second {{< include file >}}` after _outside_.
 
-- Before `first  
-    second {{< include file >}}` after *outside*.
+- Before
+  `first
+    second {{< include file >}}` after _outside_.
 
-> Before `first  
->   second {{< include file >}}` after *outside*.
+> Before
+> `first
+>   second {{< include file >}}` after _outside_.
 
-Before
-`[x]( url ) {{ value }}` after _outside_.
+Before `[x]( url ) {{ value }}` after *outside*.
 
-Before `first  
+Before `first
   {{ value }} second` after _outside_.
 
 - {{ render_item() }}
@@ -19,8 +21,8 @@
 > {{ render_item() }}
 > Follow-up prose.
 
-- Before <kbd><span>`first	
-    second {{< include file >}}`</span></kbd> after *outside* [real](target).
+- Before <kbd><span>`first
+    second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).
 
 first second `a b` $c@@ -34,24 +36,24 @@
 b` $c d$
 
-- Before `first  
-    second {{< include file >}}` after *outside*.
+- Before `first
+    second {{< include file >}}` after _outside_.
 
-  Next paragraph [real](target).
+  Next paragraph [real](  target  ).
 
 > - First paragraph `a  b`.
 >
 >   Next paragraph [real](target).
 
 [^nested]:
-    Before `first  
+    Before `first
       second {{< include file >}}` after *outside*.
 
     Next paragraph [real](target).
 
 Term
    ~ Before $first\
-     second [x]( url )$ after *outside* [real](target).
+    second [x]( url )$ after _outside_ [real](  target  ).
 
-1000) Before `first	
-      second {{< include file >}}` after *outside*.
+1000) Before `first
+    second {{< include file >}}` after _outside_.
```

```json
{
  "stdin": "Before\n`first  \n  second {{< include file >}}` after _outside_.\n\n- Before\n  `first  \n    second {{< include file >}}` after _outside_.\n\n> Before\n> `first  \n>   second {{< include file >}}` after _outside_.\n\nBefore\n`[x]( url ) {{ value }}` after _outside_.\n\nBefore `first  \n  {{ value }} second` after _outside_.\n\n- {{ render_item() }}\n  Follow-up prose.\n\n> {{ render_item() }}\n> Follow-up prose.\n\n- Before <kbd><span>`first\t\n    second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).\n\nfirst\fsecond `a\fb` $c\fd$\n\n- first\fsecond `a\fb` $c\fd$\n\n> first\fsecond `a\fb` $c\fd$\n\n- Before `first  \n    second {{< include file >}}` after _outside_.\n  \t\n  Next paragraph [real](  target  ).\n\n> - First paragraph `a  b`.\n>   \n>   Next paragraph [real](  target  ).\n\n[^nested]: Before `first  \n      second {{< include file >}}` after _outside_.\n  \t\n    Next paragraph [real](  target  ).\n\nTerm\n   ~ Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n\n1000) Before `first\t\n    second {{< include file >}}` after _outside_.\n",
  "reviewed_stdout": "Before `first  \n  second {{< include file >}}` after *outside*.\n\n- Before `first  \n    second {{< include file >}}` after *outside*.\n\n> Before `first  \n>   second {{< include file >}}` after *outside*.\n\nBefore\n`[x]( url ) {{ value }}` after _outside_.\n\nBefore `first  \n  {{ value }} second` after _outside_.\n\n- {{ render_item() }}\n  Follow-up prose.\n\n> {{ render_item() }}\n> Follow-up prose.\n\n- Before <kbd><span>`first\t\n    second {{< include file >}}`</span></kbd> after *outside* [real](target).\n\nfirst second `a\fb` $c\fd$\n\n- first second `a\fb` $c\fd$\n\n> first second `a\fb` $c\fd$\n\n- Before `first  \n    second {{< include file >}}` after *outside*.\n\n  Next paragraph [real](target).\n\n> - First paragraph `a  b`.\n>\n>   Next paragraph [real](target).\n\n[^nested]:\n    Before `first  \n      second {{< include file >}}` after *outside*.\n\n    Next paragraph [real](target).\n\nTerm\n   ~ Before $first\\\n     second [x]( url )$ after *outside* [real](target).\n\n1000) Before `first\t\n      second {{< include file >}}` after *outside*.\n",
  "current_stdout": "Before\n`first\n  second {{< include file >}}` after _outside_.\n\n- Before\n  `first\n    second {{< include file >}}` after _outside_.\n\n> Before\n> `first\n>   second {{< include file >}}` after _outside_.\n\nBefore `[x]( url ) {{ value }}` after *outside*.\n\nBefore `first\n  {{ value }} second` after _outside_.\n\n- {{ render_item() }}\n  Follow-up prose.\n\n> {{ render_item() }}\n> Follow-up prose.\n\n- Before <kbd><span>`first\n    second {{< include file >}}`</span></kbd> after _outside_ [real](  target  ).\n\nfirst second `a\fb` $c\fd$\n\n- first second `a\fb` $c\fd$\n\n> first second `a\fb` $c\fd$\n\n- Before `first\n    second {{< include file >}}` after _outside_.\n\n  Next paragraph [real](  target  ).\n\n> - First paragraph `a  b`.\n>\n>   Next paragraph [real](target).\n\n[^nested]:\n    Before `first\n      second {{< include file >}}` after *outside*.\n\n    Next paragraph [real](target).\n\nTerm\n   ~ Before $first\\\n    second [x]( url )$ after _outside_ [real](  target  ).\n\n1000) Before `first\n    second {{< include file >}}` after _outside_.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr6-883

Remaining literal/canonicalization/container behavior from #6. Some HTML rows also include the intentional #8 raw-angle stop; combined transcripts can also include #9 reflow.

[Mapping](mapping.md#pr6-883) · [active transcript](../../cases/recovered_pr6_markdown_inline_preservation_sentence.case) · [reviewed transcript](recovered_pr6_markdown_inline_preservation_sentence.case)

```text
format --stdin-file-path input.md --wrap sentence
```

```diff
--- reviewed stdout
+++ main stdout
@@ -30,13 +30,13 @@
 ` after.
 
 first-second _outside_ [real](  url  ).
+second _outside_ [real](url).
 
 first\-second _outside_ [real](  url  ).
+second _outside_ [real](url).
 
 Term
    : Before after [real](target).
 
-100. Before `first  
-     second [x]( url )` after [real](target).
+100. Before `first
+    second [x]( url )` after [real](  target  ).
```

```json
{
  "stdin": "Before\n`[x](< url >)` after.\n\nBefore\n`[x](  url  )` after [real](  target  ).\n\nBefore\n$[x](  url  )$ after [real](  target  ).\n\n- Before\n  `[x](  url  )` after [real](  target  ).\n\n> Before\n> $[x](  url  )$ after [real](  target  ).\n\nBefore\n`[x]( url ) {{< include file >}}` after.\n\nfirst\fsecond `a\fb` $c\fd$\n\n- first\fsecond `a\fb` $c\fd$\n\n> first\fsecond `a\fb` $c\fd$\n\nfirst\\   second `[x](  url  ) \\   ` after.\n\n- first\\\tsecond `[x](  url  ) \\\t` after.\n\n> first\\\fsecond `[x](  url  ) \\\f` after.\n\nfirst\u000bsecond _outside_ [real](  url  ).\n\nfirst\\\u000bsecond _outside_ [real](  url  ).\n\nTerm\n   : Before\n    after [real](  target  ).\n\n100. Before `first  \n    second [x]( url )` after [real](  target  ).\n",
  "reviewed_stdout": "Before `[x](< url >)` after.\n\nBefore `[x](  url  )` after [real](target).\n\nBefore $[x](  url  )$ after [real](target).\n\n- Before `[x](  url  )` after [real](target).\n\n> Before $[x](  url  )$ after [real](target).\n\nBefore `[x]( url ) {{< include file >}}` after.\n\nfirst second `a\fb` $c\fd$\n\n- first second `a\fb` $c\fd$\n\n> first second `a\fb` $c\fd$\n\nfirst\\ second `[x](  url  ) \\   ` after.\n\n- first\\ second `[x](  url  ) \\\t` after.\n\n> first\\ second `[x](  url  ) \\\f` after.\n\nfirst\u000bsecond _outside_ [real](  url  ).\n\nfirst\\\u000bsecond _outside_ [real](  url  ).\n\nTerm\n   : Before after [real](target).\n\n100. Before `first  \n     second [x]( url )` after [real](target).\n",
  "current_stdout": "Before `[x](< url >)` after.\n\nBefore `[x](  url  )` after [real](target).\n\nBefore $[x](  url  )$ after [real](target).\n\n- Before `[x](  url  )` after [real](target).\n\n> Before $[x](  url  )$ after [real](target).\n\nBefore `[x]( url ) {{< include file >}}` after.\n\nfirst second `a\fb` $c\fd$\n\n- first second `a\fb` $c\fd$\n\n> first second `a\fb` $c\fd$\n\nfirst\\ second `[x](  url  ) \\   ` after.\n\n- first\\ second `[x](  url  ) \\\t` after.\n\n> first\\ second `[x](  url  ) \\\f` after.\n\nfirst\u000bsecond _outside_ [real](url).\n\nfirst\\\u000bsecond _outside_ [real](url).\n\nTerm\n   : Before after [real](target).\n\n100. Before `first\n    second [x]( url )` after [real](  target  ).\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

