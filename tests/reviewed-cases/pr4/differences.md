# Exact reviewed/current differences

Historical expected output was reproduced with PR #4 head `cfbf776d8f5a318e19994a08d3ecabaf0ff16dc0`. Current output is from `0b1f072e710daa58ce4da412d8efc68f7d0fcd20`. All invocations below returned status 0 and empty stderr on both revisions.

Each command in an entry uses the same input and expectations. Escaped strings use JSON notation; `\r`, `\n`, `\t`, `\u000b`, and `\u000c` retain the exact UTF-8 bytes. These are evidence, not executable expectations.

## pr4-001

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-001) · [active transcript](../../cases/recovered_pr4_template_prose_boundaries_001.case) · [reviewed transcript](recovered_pr4_template_prose_boundaries_001.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,5 @@
-> Before this {{ foo }} after.
+> Before
+> this {{ foo }} after.
 
 Before
 <span>{{ foo }}</span> after.
```

```json
{
  "stdin": "> Before\n> this {{ foo }} after.\n\nBefore\n<span>{{ foo }}</span> after.\n\nBefore\n{{ foo }}\nafter.\n\nBefore\n{{ foo }} after.\n\nFollowing\nprose.\n",
  "reviewed_stdout": "> Before this {{ foo }} after.\n\nBefore\n<span>{{ foo }}</span> after.\n\nBefore\n{{ foo }}\nafter.\n\nBefore {{ foo }} after.\n\nFollowing prose.\n",
  "current_stdout": "> Before\n> this {{ foo }} after.\n\nBefore\n<span>{{ foo }}</span> after.\n\nBefore\n{{ foo }}\nafter.\n\nBefore {{ foo }} after.\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-058

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-058) · [active transcript](../../cases/recovered_pr4_inline_shortcodes_keep_quoted_delimiters_058.case) · [reviewed transcript](recovered_pr4_inline_shortcodes_keep_quoted_delimiters_058.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-Before {{< note text="}} keep   _this_" >}} after.
+Before   {{< note text="}} keep   _this_" >}} after.
```

```json
{
  "stdin": "Before   {{< note text=\"}} keep   _this_\" >}} after.\n",
  "reviewed_stdout": "Before {{< note text=\"}} keep   _this_\" >}} after.\n",
  "current_stdout": "Before   {{< note text=\"}} keep   _this_\" >}} after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-060

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-060) · [active transcript](../../cases/recovered_pr4_inline_shortcodes_keep_quoted_delimiters_060.case) · [reviewed transcript](recovered_pr4_inline_shortcodes_keep_quoted_delimiters_060.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-Before {{% note text="}} keep   _this_" %}} after.
+Before   {{% note text="}} keep   _this_" %}} after.
```

```json
{
  "stdin": "Before   {{% note text=\"}} keep   _this_\" %}} after.\n",
  "reviewed_stdout": "Before {{% note text=\"}} keep   _this_\" %}} after.\n",
  "current_stdout": "Before   {{% note text=\"}} keep   _this_\" %}} after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-062

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-062) · [active transcript](../../cases/recovered_pr4_inline_shortcodes_keep_quoted_delimiters_062.case) · [reviewed transcript](recovered_pr4_inline_shortcodes_keep_quoted_delimiters_062.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-Before {{< note text='}} keep   _this_' >}} after.
+Before   {{< note text='}} keep   _this_' >}} after.
```

```json
{
  "stdin": "Before   {{< note text='}} keep   _this_' >}} after.\n",
  "reviewed_stdout": "Before {{< note text='}} keep   _this_' >}} after.\n",
  "current_stdout": "Before   {{< note text='}} keep   _this_' >}} after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-064

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-064) · [active transcript](../../cases/recovered_pr4_inline_shortcodes_keep_quoted_delimiters_064.case) · [reviewed transcript](recovered_pr4_inline_shortcodes_keep_quoted_delimiters_064.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-Before {{% note text='}} keep   _this_' %}} after.
+Before   {{% note text='}} keep   _this_' %}} after.
```

```json
{
  "stdin": "Before   {{% note text='}} keep   _this_' %}} after.\n",
  "reviewed_stdout": "Before {{% note text='}} keep   _this_' %}} after.\n",
  "current_stdout": "Before   {{% note text='}} keep   _this_' %}} after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-066

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-066) · [active transcript](../../cases/recovered_pr4_inline_shortcodes_keep_quoted_delimiters_066.case) · [reviewed transcript](recovered_pr4_inline_shortcodes_keep_quoted_delimiters_066.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-Before {{< note text=`}} keep   _this_` >}} after.
+Before   {{< note text=`}} keep   _this_` >}} after.
```

```json
{
  "stdin": "Before   {{< note text=`}} keep   _this_` >}} after.\n",
  "reviewed_stdout": "Before {{< note text=`}} keep   _this_` >}} after.\n",
  "current_stdout": "Before   {{< note text=`}} keep   _this_` >}} after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-068

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-068) · [active transcript](../../cases/recovered_pr4_inline_shortcodes_keep_quoted_delimiters_068.case) · [reviewed transcript](recovered_pr4_inline_shortcodes_keep_quoted_delimiters_068.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-Before {{% note text=`}} keep   _this_` %}} after.
+Before   {{% note text=`}} keep   _this_` %}} after.
```

```json
{
  "stdin": "Before   {{% note text=`}} keep   _this_` %}} after.\n",
  "reviewed_stdout": "Before {{% note text=`}} keep   _this_` %}} after.\n",
  "current_stdout": "Before   {{% note text=`}} keep   _this_` %}} after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-070

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-070) · [active transcript](../../cases/recovered_pr4_inline_shortcodes_keep_quoted_delimiters_070.case) · [reviewed transcript](recovered_pr4_inline_shortcodes_keep_quoted_delimiters_070.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
format --canonical --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1 @@
-Before
-{{< note text="}} keep   _this_" >}}
-after.
+Before   {{< note text="}} keep   _this_" >}} after.
```

```json
{
  "stdin": "Before   {{< note text=\"}} keep   _this_\" >}} after.\n",
  "reviewed_stdout": "Before\n{{< note text=\"}} keep   _this_\" >}}\nafter.\n",
  "current_stdout": "Before   {{< note text=\"}} keep   _this_\" >}} after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-072

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-072) · [active transcript](../../cases/recovered_pr4_inline_shortcodes_keep_quoted_delimiters_072.case) · [reviewed transcript](recovered_pr4_inline_shortcodes_keep_quoted_delimiters_072.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
format --canonical --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1 @@
-Before
-{{% note text="}} keep   _this_" %}}
-after.
+Before   {{% note text="}} keep   _this_" %}} after.
```

```json
{
  "stdin": "Before   {{% note text=\"}} keep   _this_\" %}} after.\n",
  "reviewed_stdout": "Before\n{{% note text=\"}} keep   _this_\" %}}\nafter.\n",
  "current_stdout": "Before   {{% note text=\"}} keep   _this_\" %}} after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-074

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-074) · [active transcript](../../cases/recovered_pr4_inline_shortcodes_keep_quoted_delimiters_074.case) · [reviewed transcript](recovered_pr4_inline_shortcodes_keep_quoted_delimiters_074.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
format --canonical --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1 @@
-Before
-{{< note text='}} keep   _this_' >}}
-after.
+Before   {{< note text='}} keep   _this_' >}} after.
```

```json
{
  "stdin": "Before   {{< note text='}} keep   _this_' >}} after.\n",
  "reviewed_stdout": "Before\n{{< note text='}} keep   _this_' >}}\nafter.\n",
  "current_stdout": "Before   {{< note text='}} keep   _this_' >}} after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-076

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-076) · [active transcript](../../cases/recovered_pr4_inline_shortcodes_keep_quoted_delimiters_076.case) · [reviewed transcript](recovered_pr4_inline_shortcodes_keep_quoted_delimiters_076.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
format --canonical --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1 @@
-Before
-{{% note text='}} keep   _this_' %}}
-after.
+Before   {{% note text='}} keep   _this_' %}} after.
```

```json
{
  "stdin": "Before   {{% note text='}} keep   _this_' %}} after.\n",
  "reviewed_stdout": "Before\n{{% note text='}} keep   _this_' %}}\nafter.\n",
  "current_stdout": "Before   {{% note text='}} keep   _this_' %}} after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-078

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-078) · [active transcript](../../cases/recovered_pr4_inline_shortcodes_keep_quoted_delimiters_078.case) · [reviewed transcript](recovered_pr4_inline_shortcodes_keep_quoted_delimiters_078.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
format --canonical --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1 @@
-Before
-{{< note text=`}} keep   _this_` >}}
-after.
+Before   {{< note text=`}} keep   _this_` >}} after.
```

```json
{
  "stdin": "Before   {{< note text=`}} keep   _this_` >}} after.\n",
  "reviewed_stdout": "Before\n{{< note text=`}} keep   _this_` >}}\nafter.\n",
  "current_stdout": "Before   {{< note text=`}} keep   _this_` >}} after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-080

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-080) · [active transcript](../../cases/recovered_pr4_inline_shortcodes_keep_quoted_delimiters_080.case) · [reviewed transcript](recovered_pr4_inline_shortcodes_keep_quoted_delimiters_080.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
format --canonical --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1 @@
-Before
-{{% note text=`}} keep   _this_` %}}
-after.
+Before   {{% note text=`}} keep   _this_` %}} after.
```

```json
{
  "stdin": "Before   {{% note text=`}} keep   _this_` %}} after.\n",
  "reviewed_stdout": "Before\n{{% note text=`}} keep   _this_` %}}\nafter.\n",
  "current_stdout": "Before   {{% note text=`}} keep   _this_` %}} after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-090

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-090) · [active transcript](../../cases/recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_090.case) · [reviewed transcript](recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_090.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-`{{ keep  
+`{{ keep
 this }}`
 
 Following prose.
```

```json
{
  "stdin": "`{{ keep  \nthis }}`\n\nFollowing\nprose.\n",
  "reviewed_stdout": "`{{ keep  \nthis }}`\n\nFollowing prose.\n",
  "current_stdout": "`{{ keep\nthis }}`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-091

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-091)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-`{{ keep  
+`{{ keep
 this }}`
 
 Following prose.
```

```json
{
  "stdin": "`{{ keep  \nthis }}`\n\nFollowing prose.\n",
  "reviewed_stdout": "`{{ keep  \nthis }}`\n\nFollowing prose.\n",
  "current_stdout": "`{{ keep\nthis }}`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-094

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-094) · [active transcript](../../cases/recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_094.case) · [reviewed transcript](recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_094.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-`before  
+`before
 {{ foo }}`
 
 Following prose.
```

```json
{
  "stdin": "`before  \n{{ foo }}`\n\nFollowing\nprose.\n",
  "reviewed_stdout": "`before  \n{{ foo }}`\n\nFollowing prose.\n",
  "current_stdout": "`before\n{{ foo }}`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-095

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-095)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-`before  
+`before
 {{ foo }}`
 
 Following prose.
```

```json
{
  "stdin": "`before  \n{{ foo }}`\n\nFollowing prose.\n",
  "reviewed_stdout": "`before  \n{{ foo }}`\n\nFollowing prose.\n",
  "current_stdout": "`before\n{{ foo }}`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-096

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-096) · [active transcript](../../cases/recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_096.case) · [reviewed transcript](recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_096.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 `{{ foo }}
-after  
+after
 code`
 
 Following prose.
```

```json
{
  "stdin": "`{{ foo }}\nafter  \ncode`\n\nFollowing\nprose.\n",
  "reviewed_stdout": "`{{ foo }}\nafter  \ncode`\n\nFollowing prose.\n",
  "current_stdout": "`{{ foo }}\nafter\ncode`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-097

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-097)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 `{{ foo }}
-after  
+after
 code`
 
 Following prose.
```

```json
{
  "stdin": "`{{ foo }}\nafter  \ncode`\n\nFollowing prose.\n",
  "reviewed_stdout": "`{{ foo }}\nafter  \ncode`\n\nFollowing prose.\n",
  "current_stdout": "`{{ foo }}\nafter\ncode`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-098

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-098) · [active transcript](../../cases/recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_098.case) · [reviewed transcript](recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_098.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-Before {{ foo }}  
+Before {{ foo }}
 after.
 
 Following prose.
```

```json
{
  "stdin": "Before {{ foo }}  \nafter.\n\nFollowing\nprose.\n",
  "reviewed_stdout": "Before {{ foo }}  \nafter.\n\nFollowing prose.\n",
  "current_stdout": "Before {{ foo }}\nafter.\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-099

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-099)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-Before {{ foo }}  
+Before {{ foo }}
 after.
 
 Following prose.
```

```json
{
  "stdin": "Before {{ foo }}  \nafter.\n\nFollowing prose.\n",
  "reviewed_stdout": "Before {{ foo }}  \nafter.\n\nFollowing prose.\n",
  "current_stdout": "Before {{ foo }}\nafter.\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-100

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-100) · [active transcript](../../cases/recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_100.case) · [reviewed transcript](recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_100.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-> `{{ keep  
+> `{{ keep
 > this }}`
 
 Following prose.
```

```json
{
  "stdin": "> `{{ keep  \n> this }}`\n\nFollowing\nprose.\n",
  "reviewed_stdout": "> `{{ keep  \n> this }}`\n\nFollowing prose.\n",
  "current_stdout": "> `{{ keep\n> this }}`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-101

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-101)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-> `{{ keep  
+> `{{ keep
 > this }}`
 
 Following prose.
```

```json
{
  "stdin": "> `{{ keep  \n> this }}`\n\nFollowing prose.\n",
  "reviewed_stdout": "> `{{ keep  \n> this }}`\n\nFollowing prose.\n",
  "current_stdout": "> `{{ keep\n> this }}`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-104

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-104) · [active transcript](../../cases/recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_104.case) · [reviewed transcript](recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_104.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-> `before  
+> `before
 > {{ foo }}`
 
 Following prose.
```

```json
{
  "stdin": "> `before  \n> {{ foo }}`\n\nFollowing\nprose.\n",
  "reviewed_stdout": "> `before  \n> {{ foo }}`\n\nFollowing prose.\n",
  "current_stdout": "> `before\n> {{ foo }}`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-105

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-105)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-> `before  
+> `before
 > {{ foo }}`
 
 Following prose.
```

```json
{
  "stdin": "> `before  \n> {{ foo }}`\n\nFollowing prose.\n",
  "reviewed_stdout": "> `before  \n> {{ foo }}`\n\nFollowing prose.\n",
  "current_stdout": "> `before\n> {{ foo }}`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-106

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-106) · [active transcript](../../cases/recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_106.case) · [reviewed transcript](recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_106.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 > `{{ foo }}
-> after  
+> after
 > code`
 
 Following prose.
```

```json
{
  "stdin": "> `{{ foo }}\n> after  \n> code`\n\nFollowing\nprose.\n",
  "reviewed_stdout": "> `{{ foo }}\n> after  \n> code`\n\nFollowing prose.\n",
  "current_stdout": "> `{{ foo }}\n> after\n> code`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-107

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-107)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 > `{{ foo }}
-> after  
+> after
 > code`
 
 Following prose.
```

```json
{
  "stdin": "> `{{ foo }}\n> after  \n> code`\n\nFollowing prose.\n",
  "reviewed_stdout": "> `{{ foo }}\n> after  \n> code`\n\nFollowing prose.\n",
  "current_stdout": "> `{{ foo }}\n> after\n> code`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-108

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-108) · [active transcript](../../cases/recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_108.case) · [reviewed transcript](recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_108.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-> Before {{ foo }}  
+> Before {{ foo }}
 > after.
 
 Following prose.
```

```json
{
  "stdin": "> Before {{ foo }}  \n> after.\n\nFollowing\nprose.\n",
  "reviewed_stdout": "> Before {{ foo }}  \n> after.\n\nFollowing prose.\n",
  "current_stdout": "> Before {{ foo }}\n> after.\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-109

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-109)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-> Before {{ foo }}  
+> Before {{ foo }}
 > after.
 
 Following prose.
```

```json
{
  "stdin": "> Before {{ foo }}  \n> after.\n\nFollowing prose.\n",
  "reviewed_stdout": "> Before {{ foo }}  \n> after.\n\nFollowing prose.\n",
  "current_stdout": "> Before {{ foo }}\n> after.\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-110

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-110) · [active transcript](../../cases/recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_110.case) · [reviewed transcript](recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_110.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- `{{ keep  
+- `{{ keep
   this }}`
 
 Following prose.
```

```json
{
  "stdin": "- `{{ keep  \n  this }}`\n\nFollowing\nprose.\n",
  "reviewed_stdout": "- `{{ keep  \n  this }}`\n\nFollowing prose.\n",
  "current_stdout": "- `{{ keep\n  this }}`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-111

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-111)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- `{{ keep  
+- `{{ keep
   this }}`
 
 Following prose.
```

```json
{
  "stdin": "- `{{ keep  \n  this }}`\n\nFollowing prose.\n",
  "reviewed_stdout": "- `{{ keep  \n  this }}`\n\nFollowing prose.\n",
  "current_stdout": "- `{{ keep\n  this }}`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-114

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-114) · [active transcript](../../cases/recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_114.case) · [reviewed transcript](recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_114.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- `before  
+- `before
   {{ foo }}`
 
 Following prose.
```

```json
{
  "stdin": "- `before  \n  {{ foo }}`\n\nFollowing\nprose.\n",
  "reviewed_stdout": "- `before  \n  {{ foo }}`\n\nFollowing prose.\n",
  "current_stdout": "- `before\n  {{ foo }}`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-115

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-115)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- `before  
+- `before
   {{ foo }}`
 
 Following prose.
```

```json
{
  "stdin": "- `before  \n  {{ foo }}`\n\nFollowing prose.\n",
  "reviewed_stdout": "- `before  \n  {{ foo }}`\n\nFollowing prose.\n",
  "current_stdout": "- `before\n  {{ foo }}`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-116

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-116) · [active transcript](../../cases/recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_116.case) · [reviewed transcript](recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_116.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 - `{{ foo }}
-  after  
+  after
   code`
 
 Following prose.
```

```json
{
  "stdin": "- `{{ foo }}\n  after  \n  code`\n\nFollowing\nprose.\n",
  "reviewed_stdout": "- `{{ foo }}\n  after  \n  code`\n\nFollowing prose.\n",
  "current_stdout": "- `{{ foo }}\n  after\n  code`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-117

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-117)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 - `{{ foo }}
-  after  
+  after
   code`
 
 Following prose.
```

```json
{
  "stdin": "- `{{ foo }}\n  after  \n  code`\n\nFollowing prose.\n",
  "reviewed_stdout": "- `{{ foo }}\n  after  \n  code`\n\nFollowing prose.\n",
  "current_stdout": "- `{{ foo }}\n  after\n  code`\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-118

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-118) · [active transcript](../../cases/recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_118.case) · [reviewed transcript](recovered_pr4_template_blocks_with_apparent_hard_breaks_are_preserved_118.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- Before {{ foo }}  
+- Before {{ foo }}
   after.
 
 Following prose.
```

```json
{
  "stdin": "- Before {{ foo }}  \n  after.\n\nFollowing\nprose.\n",
  "reviewed_stdout": "- Before {{ foo }}  \n  after.\n\nFollowing prose.\n",
  "current_stdout": "- Before {{ foo }}\n  after.\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-119

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-119)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,4 @@
-- Before {{ foo }}  
+- Before {{ foo }}
   after.
 
 Following prose.
```

```json
{
  "stdin": "- Before {{ foo }}  \n  after.\n\nFollowing prose.\n",
  "reviewed_stdout": "- Before {{ foo }}  \n  after.\n\nFollowing prose.\n",
  "current_stdout": "- Before {{ foo }}\n  after.\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-137

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-137) · [active transcript](../../cases/recovered_pr4_braced_fig_alt_values_are_not_split_137.case) · [reviewed transcript](recovered_pr4_braced_fig_alt_values_are_not_split_137.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1 @@
-![x](url){
-  fig-alt="{{ 'keep this' }}"
-}
+![x](url){fig-alt="{{ 'keep this' }}"}
```

```json
{
  "stdin": "![x](url){fig-alt=\"{{ 'keep this' }}\"}\n",
  "reviewed_stdout": "![x](url){\n  fig-alt=\"{{ 'keep this' }}\"\n}\n",
  "current_stdout": "![x](url){fig-alt=\"{{ 'keep this' }}\"}\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-139

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-139) · [active transcript](../../cases/recovered_pr4_braced_fig_alt_values_are_not_split_139.case) · [reviewed transcript](recovered_pr4_braced_fig_alt_values_are_not_split_139.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1 @@
-![x](url){
-  fig-alt="one {{ 'keep this' }} two"
-}
+![x](url){fig-alt="one {{ 'keep this' }} two"}
```

```json
{
  "stdin": "![x](url){fig-alt=\"one {{ 'keep this' }} two\"}\n",
  "reviewed_stdout": "![x](url){\n  fig-alt=\"one {{ 'keep this' }} two\"\n}\n",
  "current_stdout": "![x](url){fig-alt=\"one {{ 'keep this' }} two\"}\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-141

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-141) · [active transcript](../../cases/recovered_pr4_attached_templates_stay_in_one_token_141.case) · [reviewed transcript](recovered_pr4_attached_templates_stay_in_one_token_141.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with prefix{{ "keep   this" }}.
-Second sentence.
+First
+sentence with prefix{{ "keep   this" }}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with prefix{{ \"keep   this\" }}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with prefix{{ \"keep   this\" }}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with prefix{{ \"keep   this\" }}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-143

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-143) · [active transcript](../../cases/recovered_pr4_attached_templates_stay_in_one_token_143.case) · [reviewed transcript](recovered_pr4_attached_templates_stay_in_one_token_143.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with prefix{% set x = "keep   this" %}.
-Second sentence.
+First
+sentence with prefix{% set x = "keep   this" %}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with prefix{% set x = \"keep   this\" %}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with prefix{% set x = \"keep   this\" %}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with prefix{% set x = \"keep   this\" %}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-145

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-145) · [active transcript](../../cases/recovered_pr4_attached_templates_stay_in_one_token_145.case) · [reviewed transcript](recovered_pr4_attached_templates_stay_in_one_token_145.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with prefix{{ "keep   this" }}suffix.
-Second sentence.
+First
+sentence with prefix{{ "keep   this" }}suffix. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with prefix{{ \"keep   this\" }}suffix. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with prefix{{ \"keep   this\" }}suffix.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with prefix{{ \"keep   this\" }}suffix. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-147

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-147) · [active transcript](../../cases/recovered_pr4_attached_templates_stay_in_one_token_147.case) · [reviewed transcript](recovered_pr4_attached_templates_stay_in_one_token_147.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with prefix{% set x = "keep   this" %}suffix.
-Second sentence.
+First
+sentence with prefix{% set x = "keep   this" %}suffix. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with prefix{% set x = \"keep   this\" %}suffix. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with prefix{% set x = \"keep   this\" %}suffix.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with prefix{% set x = \"keep   this\" %}suffix. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-149

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-149) · [active transcript](../../cases/recovered_pr4_attached_templates_stay_in_one_token_149.case) · [reviewed transcript](recovered_pr4_attached_templates_stay_in_one_token_149.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-First sentence with
-prefix{{ "keep   this" }}.
-Second sentence.
+First
+sentence with prefix{{ "keep   this" }}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with prefix{{ \"keep   this\" }}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with\nprefix{{ \"keep   this\" }}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with prefix{{ \"keep   this\" }}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-151

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-151) · [active transcript](../../cases/recovered_pr4_attached_templates_stay_in_one_token_151.case) · [reviewed transcript](recovered_pr4_attached_templates_stay_in_one_token_151.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-First sentence with
-prefix{% set x = "keep   this" %}.
-Second sentence.
+First
+sentence with prefix{% set x = "keep   this" %}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with prefix{% set x = \"keep   this\" %}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with\nprefix{% set x = \"keep   this\" %}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with prefix{% set x = \"keep   this\" %}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-153

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-153) · [active transcript](../../cases/recovered_pr4_attached_templates_stay_in_one_token_153.case) · [reviewed transcript](recovered_pr4_attached_templates_stay_in_one_token_153.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-First sentence with
-prefix{{ "keep   this" }}suffix.
-Second sentence.
+First
+sentence with prefix{{ "keep   this" }}suffix. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with prefix{{ \"keep   this\" }}suffix. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with\nprefix{{ \"keep   this\" }}suffix.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with prefix{{ \"keep   this\" }}suffix. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-155

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-155) · [active transcript](../../cases/recovered_pr4_attached_templates_stay_in_one_token_155.case) · [reviewed transcript](recovered_pr4_attached_templates_stay_in_one_token_155.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-First sentence with
-prefix{% set x = "keep   this" %}suffix.
-Second sentence.
+First
+sentence with prefix{% set x = "keep   this" %}suffix. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with prefix{% set x = \"keep   this\" %}suffix. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with\nprefix{% set x = \"keep   this\" %}suffix.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with prefix{% set x = \"keep   this\" %}suffix. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-185

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-185) · [active transcript](../../cases/recovered_pr4_template_tag_names_do_not_change_markdown_formatting_185.case) · [reviewed transcript](recovered_pr4_template_tag_names_do_not_change_markdown_formatting_185.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1,3 @@
-Before {% raw %}keep this{% endraw %} after.
+Before
+{% raw %}keep   this{% endraw %}
+after.
```

```json
{
  "stdin": "Before\n{% raw %}keep   this{% endraw %}\nafter.\n",
  "reviewed_stdout": "Before {% raw %}keep this{% endraw %} after.\n",
  "current_stdout": "Before\n{% raw %}keep   this{% endraw %}\nafter.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-187

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-187) · [active transcript](../../cases/recovered_pr4_template_tag_names_do_not_change_markdown_formatting_187.case) · [reviewed transcript](recovered_pr4_template_tag_names_do_not_change_markdown_formatting_187.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1,3 @@
-Before {% verbatim %}keep this{% endverbatim %} after.
+Before
+{% verbatim %}keep   this{% endverbatim %}
+after.
```

```json
{
  "stdin": "Before\n{% verbatim %}keep   this{% endverbatim %}\nafter.\n",
  "reviewed_stdout": "Before {% verbatim %}keep this{% endverbatim %} after.\n",
  "current_stdout": "Before\n{% verbatim %}keep   this{% endverbatim %}\nafter.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-189

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-189) · [active transcript](../../cases/recovered_pr4_template_tag_names_do_not_change_markdown_formatting_189.case) · [reviewed transcript](recovered_pr4_template_tag_names_do_not_change_markdown_formatting_189.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1,3 @@
-Before {% verbatim example %}keep this{% endverbatim example %} after.
+Before
+{% verbatim example %}keep   this{% endverbatim example %}
+after.
```

```json
{
  "stdin": "Before\n{% verbatim example %}keep   this{% endverbatim example %}\nafter.\n",
  "reviewed_stdout": "Before {% verbatim example %}keep this{% endverbatim example %} after.\n",
  "current_stdout": "Before\n{% verbatim example %}keep   this{% endverbatim example %}\nafter.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-220

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-220) · [active transcript](../../cases/recovered_pr4_braced_templates_survive_image_attribute_normalization_220.case) · [reviewed transcript](recovered_pr4_braced_templates_survive_image_attribute_normalization_220.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-![x](url){fig-alt="{{ keep   this }}"}
+![x](url) { fig-alt="{{ keep   this }}" }
```

```json
{
  "stdin": "![x](url) { fig-alt=\"{{ keep   this }}\" }\n",
  "reviewed_stdout": "![x](url){fig-alt=\"{{ keep   this }}\"}\n",
  "current_stdout": "![x](url) { fig-alt=\"{{ keep   this }}\" }\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-222

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-222) · [active transcript](../../cases/recovered_pr4_braced_templates_survive_image_attribute_normalization_222.case) · [reviewed transcript](recovered_pr4_braced_templates_survive_image_attribute_normalization_222.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-![x](url){fig-alt="`{{ keep   this }}`"}
+![x](url) { fig-alt="`{{ keep   this }}`" }
```

```json
{
  "stdin": "![x](url) { fig-alt=\"`{{ keep   this }}`\" }\n",
  "reviewed_stdout": "![x](url){fig-alt=\"`{{ keep   this }}`\"}\n",
  "current_stdout": "![x](url) { fig-alt=\"`{{ keep   this }}`\" }\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-228

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-228) · [active transcript](../../cases/recovered_pr4_quoted_braces_stay_inside_the_template_228.case) · [reviewed transcript](recovered_pr4_quoted_braces_stay_inside_the_template_228.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with {{ "}}" ~ "keep   this" }}.
-Second sentence.
+First
+sentence with {{ "}}" ~ "keep   this" }}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with {{ \"}}\" ~ \"keep   this\" }}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with {{ \"}}\" ~ \"keep   this\" }}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with {{ \"}}\" ~ \"keep   this\" }}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-230

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-230) · [active transcript](../../cases/recovered_pr4_quoted_braces_stay_inside_the_template_230.case) · [reviewed transcript](recovered_pr4_quoted_braces_stay_inside_the_template_230.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with {{ '}}' ~ 'keep   this' }}.
-Second sentence.
+First
+sentence with {{ '}}' ~ 'keep   this' }}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with {{ '}}' ~ 'keep   this' }}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with {{ '}}' ~ 'keep   this' }}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with {{ '}}' ~ 'keep   this' }}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-232

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-232) · [active transcript](../../cases/recovered_pr4_quoted_braces_stay_inside_the_template_232.case) · [reviewed transcript](recovered_pr4_quoted_braces_stay_inside_the_template_232.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with {{ "escaped \"}}" ~ "keep   this" }}.
-Second sentence.
+First
+sentence with {{ "escaped \"}}" ~ "keep   this" }}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with {{ \"escaped \\\"}}\" ~ \"keep   this\" }}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with {{ \"escaped \\\"}}\" ~ \"keep   this\" }}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with {{ \"escaped \\\"}}\" ~ \"keep   this\" }}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-234

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-234) · [active transcript](../../cases/recovered_pr4_quoted_braces_stay_inside_the_template_234.case) · [reviewed transcript](recovered_pr4_quoted_braces_stay_inside_the_template_234.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with {{ render({"label": "} keep   this", "other": "{{"}) }}.
-Second sentence.
+First
+sentence with {{ render({"label": "} keep   this", "other": "{{"}) }}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with {{ render({\"label\": \"} keep   this\", \"other\": \"{{\"}) }}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with {{ render({\"label\": \"} keep   this\", \"other\": \"{{\"}) }}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with {{ render({\"label\": \"} keep   this\", \"other\": \"{{\"}) }}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-236

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-236) · [active transcript](../../cases/recovered_pr4_quoted_braces_stay_inside_the_template_236.case) · [reviewed transcript](recovered_pr4_quoted_braces_stay_inside_the_template_236.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-First sentence with
-{{ "}}" ~ "keep   this" }}.
-Second sentence.
+First
+sentence with {{ "}}" ~ "keep   this" }}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with {{ \"}}\" ~ \"keep   this\" }}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with\n{{ \"}}\" ~ \"keep   this\" }}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with {{ \"}}\" ~ \"keep   this\" }}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-238

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-238) · [active transcript](../../cases/recovered_pr4_quoted_braces_stay_inside_the_template_238.case) · [reviewed transcript](recovered_pr4_quoted_braces_stay_inside_the_template_238.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-First sentence with
-{{ '}}' ~ 'keep   this' }}.
-Second sentence.
+First
+sentence with {{ '}}' ~ 'keep   this' }}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with {{ '}}' ~ 'keep   this' }}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with\n{{ '}}' ~ 'keep   this' }}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with {{ '}}' ~ 'keep   this' }}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-240

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-240) · [active transcript](../../cases/recovered_pr4_quoted_braces_stay_inside_the_template_240.case) · [reviewed transcript](recovered_pr4_quoted_braces_stay_inside_the_template_240.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-First sentence with
-{{ "escaped \"}}" ~ "keep   this" }}.
-Second sentence.
+First
+sentence with {{ "escaped \"}}" ~ "keep   this" }}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with {{ \"escaped \\\"}}\" ~ \"keep   this\" }}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with\n{{ \"escaped \\\"}}\" ~ \"keep   this\" }}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with {{ \"escaped \\\"}}\" ~ \"keep   this\" }}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-242

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-242) · [active transcript](../../cases/recovered_pr4_quoted_braces_stay_inside_the_template_242.case) · [reviewed transcript](recovered_pr4_quoted_braces_stay_inside_the_template_242.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
-First sentence with
-{{ render({"label": "} keep   this", "other": "{{"}) }}.
-Second sentence.
+First
+sentence with {{ render({"label": "} keep   this", "other": "{{"}) }}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with {{ render({\"label\": \"} keep   this\", \"other\": \"{{\"}) }}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with\n{{ render({\"label\": \"} keep   this\", \"other\": \"{{\"}) }}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with {{ render({\"label\": \"} keep   this\", \"other\": \"{{\"}) }}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-265

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-265) · [active transcript](../../cases/recovered_pr4_containers_preserve_multiline_code_265.case) · [reviewed transcript](recovered_pr4_containers_preserve_multiline_code_265.case)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,5 @@
 Leading prose.
 
-> Before `code
-> keep   _this_` after.
+> Before `code keep _this_` after.
 
 Following prose.
```

```json
{
  "stdin": "Leading\nprose.\n\n> Before `code\n> keep   _this_` after.\n\nFollowing\nprose.\n",
  "reviewed_stdout": "Leading prose.\n\n> Before `code\n> keep   _this_` after.\n\nFollowing prose.\n",
  "current_stdout": "Leading prose.\n\n> Before `code keep _this_` after.\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-266

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-266)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,5 @@
 Leading prose.
 
-> Before `code
-> keep   _this_` after.
+> Before `code keep _this_` after.
 
 Following prose.
```

```json
{
  "stdin": "Leading prose.\n\n> Before `code\n> keep   _this_` after.\n\nFollowing prose.\n",
  "reviewed_stdout": "Leading prose.\n\n> Before `code\n> keep   _this_` after.\n\nFollowing prose.\n",
  "current_stdout": "Leading prose.\n\n> Before `code keep _this_` after.\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-267

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-267) · [active transcript](../../cases/recovered_pr4_containers_preserve_multiline_code_267.case) · [reviewed transcript](recovered_pr4_containers_preserve_multiline_code_267.case)

```text
format --canonical --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,7 @@
 Leading prose.
 
-> Before `code
-> keep   _this_` after.
+> Before
+> `code keep _this_`
+> after.
 
 Following prose.
```

```json
{
  "stdin": "Leading\nprose.\n\n> Before `code\n> keep   _this_` after.\n\nFollowing\nprose.\n",
  "reviewed_stdout": "Leading prose.\n\n> Before `code\n> keep   _this_` after.\n\nFollowing prose.\n",
  "current_stdout": "Leading prose.\n\n> Before\n> `code keep _this_`\n> after.\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-268

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-268)

```text
format --canonical --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,7 @@
 Leading prose.
 
-> Before `code
-> keep   _this_` after.
+> Before
+> `code keep _this_`
+> after.
 
 Following prose.
```

```json
{
  "stdin": "Leading prose.\n\n> Before `code\n> keep   _this_` after.\n\nFollowing prose.\n",
  "reviewed_stdout": "Leading prose.\n\n> Before `code\n> keep   _this_` after.\n\nFollowing prose.\n",
  "current_stdout": "Leading prose.\n\n> Before\n> `code keep _this_`\n> after.\n\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-281

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-281)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,5 @@
 Leading prose.
 
-> Before `code
-> keep   _this_` after.
+> Before `code keep _this_` after.
 
 Following prose.
```

```json
{
  "stdin": "Leading\r\nprose.\r\n\r\n> Before `code\r\n> keep   _this_` after.\r\n\r\nFollowing\r\nprose.\r\n",
  "reviewed_stdout": "Leading prose.\r\n\r\n> Before `code\r\n> keep   _this_` after.\r\n\r\nFollowing prose.\r\n",
  "current_stdout": "Leading prose.\r\n\r\n> Before `code keep _this_` after.\r\n\r\nFollowing prose.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-282

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-282)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,5 @@
 Leading prose.
 
-> Before `code
-> keep   _this_` after.
+> Before `code keep _this_` after.
 
 Following prose.
```

```json
{
  "stdin": "Leading prose.\r\n\r\n> Before `code\r\n> keep   _this_` after.\r\n\r\nFollowing prose.\r\n",
  "reviewed_stdout": "Leading prose.\r\n\r\n> Before `code\r\n> keep   _this_` after.\r\n\r\nFollowing prose.\r\n",
  "current_stdout": "Leading prose.\r\n\r\n> Before `code keep _this_` after.\r\n\r\nFollowing prose.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-283

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-283)

```text
format --canonical --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,7 @@
 Leading prose.
 
-> Before `code
-> keep   _this_` after.
+> Before
+> `code keep _this_`
+> after.
 
 Following prose.
```

```json
{
  "stdin": "Leading\r\nprose.\r\n\r\n> Before `code\r\n> keep   _this_` after.\r\n\r\nFollowing\r\nprose.\r\n",
  "reviewed_stdout": "Leading prose.\r\n\r\n> Before `code\r\n> keep   _this_` after.\r\n\r\nFollowing prose.\r\n",
  "current_stdout": "Leading prose.\r\n\r\n> Before\r\n> `code keep _this_`\r\n> after.\r\n\r\nFollowing prose.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-284

Main trims literal line-end spaces or collapses ordinary multiline code. Recording this limitation does not endorse it.

[Mapping](mapping.md#pr4-284)

```text
format --canonical --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,7 @@
 Leading prose.
 
-> Before `code
-> keep   _this_` after.
+> Before
+> `code keep _this_`
+> after.
 
 Following prose.
```

```json
{
  "stdin": "Leading prose.\r\n\r\n> Before `code\r\n> keep   _this_` after.\r\n\r\nFollowing prose.\r\n",
  "reviewed_stdout": "Leading prose.\r\n\r\n> Before `code\r\n> keep   _this_` after.\r\n\r\nFollowing prose.\r\n",
  "current_stdout": "Leading prose.\r\n\r\n> Before\r\n> `code keep _this_`\r\n> after.\r\n\r\nFollowing prose.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-285

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-285) · [active transcript](../../cases/recovered_pr4_multiline_template_code_wraps_by_its_first_and_last_lines_285.case) · [reviewed transcript](recovered_pr4_multiline_template_code_wraps_by_its_first_and_last_lines_285.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,3 @@
-Before `{{
+Before   `{{
   render(customer.delivery_address)
-}}` after the
-template and more
-prose.
+}}` after the template and more prose.
```

```json
{
  "stdin": "Before   `{{\n  render(customer.delivery_address)\n}}` after the template and more prose.\n",
  "reviewed_stdout": "Before `{{\n  render(customer.delivery_address)\n}}` after the\ntemplate and more\nprose.\n",
  "current_stdout": "Before   `{{\n  render(customer.delivery_address)\n}}` after the template and more prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-287

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-287)

```text
format --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,3 @@
-Before `{{
+Before   `{{
   render(customer.delivery_address)
-}}` after the
-template and more
-prose.
+}}` after the template and more prose.
```

```json
{
  "stdin": "Before   `{{\r\n  render(customer.delivery_address)\r\n}}` after the template and more prose.\r\n",
  "reviewed_stdout": "Before `{{\r\n  render(customer.delivery_address)\r\n}}` after the\r\ntemplate and more\r\nprose.\r\n",
  "current_stdout": "Before   `{{\r\n  render(customer.delivery_address)\r\n}}` after the template and more prose.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-289

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-289) · [active transcript](../../cases/recovered_pr4_multiline_template_code_wraps_by_its_first_and_last_lines_289.case) · [reviewed transcript](recovered_pr4_multiline_template_code_wraps_by_its_first_and_last_lines_289.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,3 @@
-Before
-`{{ render_customer_notice(
+Before   `{{ render_customer_notice(
   address
-) }}` after the
-template and more
-prose.
+) }}` after the template and more prose.
```

```json
{
  "stdin": "Before   `{{ render_customer_notice(\n  address\n) }}` after the template and more prose.\n",
  "reviewed_stdout": "Before\n`{{ render_customer_notice(\n  address\n) }}` after the\ntemplate and more\nprose.\n",
  "current_stdout": "Before   `{{ render_customer_notice(\n  address\n) }}` after the template and more prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-291

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-291)

```text
format --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,6 +1,3 @@
-Before
-`{{ render_customer_notice(
+Before   `{{ render_customer_notice(
   address
-) }}` after the
-template and more
-prose.
+) }}` after the template and more prose.
```

```json
{
  "stdin": "Before   `{{ render_customer_notice(\r\n  address\r\n) }}` after the template and more prose.\r\n",
  "reviewed_stdout": "Before\r\n`{{ render_customer_notice(\r\n  address\r\n) }}` after the\r\ntemplate and more\r\nprose.\r\n",
  "current_stdout": "Before   `{{ render_customer_notice(\r\n  address\r\n) }}` after the template and more prose.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-293

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-293) · [active transcript](../../cases/recovered_pr4_multiline_template_code_wraps_by_its_first_and_last_lines_293.case) · [reviewed transcript](recovered_pr4_multiline_template_code_wraps_by_its_first_and_last_lines_293.case)

```text
format --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,2 @@
-Before `{{
-  render(customer.delivery_address) }}`
-after the template
-and more prose.
+Before   `{{
+  render(customer.delivery_address) }}` after the template and more prose.
```

```json
{
  "stdin": "Before   `{{\n  render(customer.delivery_address) }}` after the template and more prose.\n",
  "reviewed_stdout": "Before `{{\n  render(customer.delivery_address) }}`\nafter the template\nand more prose.\n",
  "current_stdout": "Before   `{{\n  render(customer.delivery_address) }}` after the template and more prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-295

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-295)

```text
format --wrap 20 --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,2 @@
-Before `{{
-  render(customer.delivery_address) }}`
-after the template
-and more prose.
+Before   `{{
+  render(customer.delivery_address) }}` after the template and more prose.
```

```json
{
  "stdin": "Before   `{{\r\n  render(customer.delivery_address) }}` after the template and more prose.\r\n",
  "reviewed_stdout": "Before `{{\r\n  render(customer.delivery_address) }}`\r\nafter the template\r\nand more prose.\r\n",
  "current_stdout": "Before   `{{\r\n  render(customer.delivery_address) }}` after the template and more prose.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-297

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-297) · [active transcript](../../cases/recovered_pr4_shortcode_tags_leave_intervening_markdown_to_format_297.case) · [reviewed transcript](recovered_pr4_shortcode_tags_leave_intervening_markdown_to_format_297.case)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,7 +1,8 @@
 {{% notice %}}
-Format this paragraph as Markdown.
+Format   this paragraph
+as Markdown.
 
-# A heading
+#   A heading
 
 {{% /notice %}}
 Following prose.
```

```json
{
  "stdin": "{{% notice %}}\nFormat   this paragraph\nas Markdown.\n\n#   A heading\n\n{{% /notice %}}\nFollowing\nprose.\n",
  "reviewed_stdout": "{{% notice %}}\nFormat this paragraph as Markdown.\n\n# A heading\n\n{{% /notice %}}\nFollowing prose.\n",
  "current_stdout": "{{% notice %}}\nFormat   this paragraph\nas Markdown.\n\n#   A heading\n\n{{% /notice %}}\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-299

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-299) · [active transcript](../../cases/recovered_pr4_shortcode_tags_leave_intervening_markdown_to_format_299.case) · [reviewed transcript](recovered_pr4_shortcode_tags_leave_intervening_markdown_to_format_299.case)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,7 +1,8 @@
 {{< notice >}}
-Format this paragraph as Markdown.
+Format   this paragraph
+as Markdown.
 
-# A heading
+#   A heading
 
 {{< /notice >}}
 Following prose.
```

```json
{
  "stdin": "{{< notice >}}\nFormat   this paragraph\nas Markdown.\n\n#   A heading\n\n{{< /notice >}}\nFollowing\nprose.\n",
  "reviewed_stdout": "{{< notice >}}\nFormat this paragraph as Markdown.\n\n# A heading\n\n{{< /notice >}}\nFollowing prose.\n",
  "current_stdout": "{{< notice >}}\nFormat   this paragraph\nas Markdown.\n\n#   A heading\n\n{{< /notice >}}\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-303

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-303) · [active transcript](../../cases/recovered_pr4_shortcode_tags_leave_intervening_markdown_to_format_303.case) · [reviewed transcript](recovered_pr4_shortcode_tags_leave_intervening_markdown_to_format_303.case)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,9 +1,10 @@
 {{% notice
 message="keep   _this_"
 %}}
-Format this paragraph as Markdown.
+Format   this paragraph
+as Markdown.
 
-# A heading
+#   A heading
 
 {{% /notice %}}
 Following prose.
```

```json
{
  "stdin": "{{% notice\nmessage=\"keep   _this_\"\n%}}\nFormat   this paragraph\nas Markdown.\n\n#   A heading\n\n{{% /notice %}}\nFollowing\nprose.\n",
  "reviewed_stdout": "{{% notice\nmessage=\"keep   _this_\"\n%}}\nFormat this paragraph as Markdown.\n\n# A heading\n\n{{% /notice %}}\nFollowing prose.\n",
  "current_stdout": "{{% notice\nmessage=\"keep   _this_\"\n%}}\nFormat   this paragraph\nas Markdown.\n\n#   A heading\n\n{{% /notice %}}\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-305

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-305) · [active transcript](../../cases/recovered_pr4_shortcode_tags_leave_intervening_markdown_to_format_305.case) · [reviewed transcript](recovered_pr4_shortcode_tags_leave_intervening_markdown_to_format_305.case)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -2,9 +2,10 @@
 message='quoted >}} keeps   its spacing'
 caption='another   argument'
 >}}
-Format this paragraph as Markdown.
+Format   this paragraph
+as Markdown.
 
-# A heading
+#   A heading
 
 {{< /notice >}}
 Following prose.
```

```json
{
  "stdin": "{{< notice\nmessage='quoted >}} keeps   its spacing'\ncaption='another   argument'\n>}}\nFormat   this paragraph\nas Markdown.\n\n#   A heading\n\n{{< /notice >}}\nFollowing\nprose.\n",
  "reviewed_stdout": "{{< notice\nmessage='quoted >}} keeps   its spacing'\ncaption='another   argument'\n>}}\nFormat this paragraph as Markdown.\n\n# A heading\n\n{{< /notice >}}\nFollowing prose.\n",
  "current_stdout": "{{< notice\nmessage='quoted >}} keeps   its spacing'\ncaption='another   argument'\n>}}\nFormat   this paragraph\nas Markdown.\n\n#   A heading\n\n{{< /notice >}}\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-307

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-307) · [active transcript](../../cases/recovered_pr4_shortcode_tags_leave_intervening_markdown_to_format_307.case) · [reviewed transcript](recovered_pr4_shortcode_tags_leave_intervening_markdown_to_format_307.case)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,7 +1,5 @@
 {{% notice
-message="quoted %}} keeps   its spacing"
-%}}
-Format this paragraph as Markdown.
+message="quoted %}} keeps its spacing" %}} Format this paragraph as Markdown.
 
 # A heading
 
```

```json
{
  "stdin": "{{% notice\nmessage=\"quoted %}} keeps   its spacing\"\n%}}\nFormat   this paragraph\nas Markdown.\n\n#   A heading\n\n{{% unrelated %}}\nFollowing\nprose.\n",
  "reviewed_stdout": "{{% notice\nmessage=\"quoted %}} keeps   its spacing\"\n%}}\nFormat this paragraph as Markdown.\n\n# A heading\n\n{{% unrelated %}}\nFollowing prose.\n",
  "current_stdout": "{{% notice\nmessage=\"quoted %}} keeps its spacing\" %}} Format this paragraph as Markdown.\n\n# A heading\n\n{{% unrelated %}}\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-308

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-308)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,7 +1,5 @@
 {{% notice
-message="quoted %}} keeps   its spacing"
-%}}
-Format this paragraph as Markdown.
+message="quoted %}} keeps its spacing" %}} Format this paragraph as Markdown.
 
 # A heading
 
```

```json
{
  "stdin": "{{% notice\nmessage=\"quoted %}} keeps   its spacing\"\n%}}\nFormat this paragraph as Markdown.\n\n# A heading\n\n{{% unrelated %}}\nFollowing prose.\n",
  "reviewed_stdout": "{{% notice\nmessage=\"quoted %}} keeps   its spacing\"\n%}}\nFormat this paragraph as Markdown.\n\n# A heading\n\n{{% unrelated %}}\nFollowing prose.\n",
  "current_stdout": "{{% notice\nmessage=\"quoted %}} keeps its spacing\" %}} Format this paragraph as Markdown.\n\n# A heading\n\n{{% unrelated %}}\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-309

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-309) · [active transcript](../../cases/recovered_pr4_shortcode_tags_leave_intervening_markdown_to_format_309.case) · [reviewed transcript](recovered_pr4_shortcode_tags_leave_intervening_markdown_to_format_309.case)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,11 +1,12 @@
 {{< notice
-message=`keep   this  
+message=`keep   this
 
 and   this`
 >}}
-Format this paragraph as Markdown.
+Format   this paragraph
+as Markdown.
 
-# A heading
+#   A heading
 
 {{< /notice >}}
 Following prose.
```

```json
{
  "stdin": "{{< notice\nmessage=`keep   this  \n\nand   this`\n>}}\nFormat   this paragraph\nas Markdown.\n\n#   A heading\n\n{{< /notice >}}\nFollowing\nprose.\n",
  "reviewed_stdout": "{{< notice\nmessage=`keep   this  \n\nand   this`\n>}}\nFormat this paragraph as Markdown.\n\n# A heading\n\n{{< /notice >}}\nFollowing prose.\n",
  "current_stdout": "{{< notice\nmessage=`keep   this\n\nand   this`\n>}}\nFormat   this paragraph\nas Markdown.\n\n#   A heading\n\n{{< /notice >}}\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-310

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-310)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 {{< notice
-message=`keep   this  
+message=`keep   this
 
 and   this`
 >}}
```

```json
{
  "stdin": "{{< notice\nmessage=`keep   this  \n\nand   this`\n>}}\nFormat this paragraph as Markdown.\n\n# A heading\n\n{{< /notice >}}\nFollowing prose.\n",
  "reviewed_stdout": "{{< notice\nmessage=`keep   this  \n\nand   this`\n>}}\nFormat this paragraph as Markdown.\n\n# A heading\n\n{{< /notice >}}\nFollowing prose.\n",
  "current_stdout": "{{< notice\nmessage=`keep   this\n\nand   this`\n>}}\nFormat this paragraph as Markdown.\n\n# A heading\n\n{{< /notice >}}\nFollowing prose.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-311

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-311)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,7 +1,8 @@
 {{% notice %}}
-Format this paragraph as Markdown.
+Format   this paragraph
+as Markdown.
 
-# A heading
+#   A heading
 
 {{% /notice %}}
 Following prose.
```

```json
{
  "stdin": "{{% notice %}}\r\nFormat   this paragraph\r\nas Markdown.\r\n\r\n#   A heading\r\n\r\n{{% /notice %}}\r\nFollowing\r\nprose.\r\n",
  "reviewed_stdout": "{{% notice %}}\r\nFormat this paragraph as Markdown.\r\n\r\n# A heading\r\n\r\n{{% /notice %}}\r\nFollowing prose.\r\n",
  "current_stdout": "{{% notice %}}\r\nFormat   this paragraph\r\nas Markdown.\r\n\r\n#   A heading\r\n\r\n{{% /notice %}}\r\nFollowing prose.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-313

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-313)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,7 +1,8 @@
 {{< notice >}}
-Format this paragraph as Markdown.
+Format   this paragraph
+as Markdown.
 
-# A heading
+#   A heading
 
 {{< /notice >}}
 Following prose.
```

```json
{
  "stdin": "{{< notice >}}\r\nFormat   this paragraph\r\nas Markdown.\r\n\r\n#   A heading\r\n\r\n{{< /notice >}}\r\nFollowing\r\nprose.\r\n",
  "reviewed_stdout": "{{< notice >}}\r\nFormat this paragraph as Markdown.\r\n\r\n# A heading\r\n\r\n{{< /notice >}}\r\nFollowing prose.\r\n",
  "current_stdout": "{{< notice >}}\r\nFormat   this paragraph\r\nas Markdown.\r\n\r\n#   A heading\r\n\r\n{{< /notice >}}\r\nFollowing prose.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-317

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-317)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,9 +1,10 @@
 {{% notice
 message="keep   _this_"
 %}}
-Format this paragraph as Markdown.
+Format   this paragraph
+as Markdown.
 
-# A heading
+#   A heading
 
 {{% /notice %}}
 Following prose.
```

```json
{
  "stdin": "{{% notice\r\nmessage=\"keep   _this_\"\r\n%}}\r\nFormat   this paragraph\r\nas Markdown.\r\n\r\n#   A heading\r\n\r\n{{% /notice %}}\r\nFollowing\r\nprose.\r\n",
  "reviewed_stdout": "{{% notice\r\nmessage=\"keep   _this_\"\r\n%}}\r\nFormat this paragraph as Markdown.\r\n\r\n# A heading\r\n\r\n{{% /notice %}}\r\nFollowing prose.\r\n",
  "current_stdout": "{{% notice\r\nmessage=\"keep   _this_\"\r\n%}}\r\nFormat   this paragraph\r\nas Markdown.\r\n\r\n#   A heading\r\n\r\n{{% /notice %}}\r\nFollowing prose.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-319

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-319)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -2,9 +2,10 @@
 message='quoted >}} keeps   its spacing'
 caption='another   argument'
 >}}
-Format this paragraph as Markdown.
+Format   this paragraph
+as Markdown.
 
-# A heading
+#   A heading
 
 {{< /notice >}}
 Following prose.
```

```json
{
  "stdin": "{{< notice\r\nmessage='quoted >}} keeps   its spacing'\r\ncaption='another   argument'\r\n>}}\r\nFormat   this paragraph\r\nas Markdown.\r\n\r\n#   A heading\r\n\r\n{{< /notice >}}\r\nFollowing\r\nprose.\r\n",
  "reviewed_stdout": "{{< notice\r\nmessage='quoted >}} keeps   its spacing'\r\ncaption='another   argument'\r\n>}}\r\nFormat this paragraph as Markdown.\r\n\r\n# A heading\r\n\r\n{{< /notice >}}\r\nFollowing prose.\r\n",
  "current_stdout": "{{< notice\r\nmessage='quoted >}} keeps   its spacing'\r\ncaption='another   argument'\r\n>}}\r\nFormat   this paragraph\r\nas Markdown.\r\n\r\n#   A heading\r\n\r\n{{< /notice >}}\r\nFollowing prose.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-321

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-321)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,7 +1,5 @@
 {{% notice
-message="quoted %}} keeps   its spacing"
-%}}
-Format this paragraph as Markdown.
+message="quoted %}} keeps its spacing" %}} Format this paragraph as Markdown.
 
 # A heading
 
```

```json
{
  "stdin": "{{% notice\r\nmessage=\"quoted %}} keeps   its spacing\"\r\n%}}\r\nFormat   this paragraph\r\nas Markdown.\r\n\r\n#   A heading\r\n\r\n{{% unrelated %}}\r\nFollowing\r\nprose.\r\n",
  "reviewed_stdout": "{{% notice\r\nmessage=\"quoted %}} keeps   its spacing\"\r\n%}}\r\nFormat this paragraph as Markdown.\r\n\r\n# A heading\r\n\r\n{{% unrelated %}}\r\nFollowing prose.\r\n",
  "current_stdout": "{{% notice\r\nmessage=\"quoted %}} keeps its spacing\" %}} Format this paragraph as Markdown.\r\n\r\n# A heading\r\n\r\n{{% unrelated %}}\r\nFollowing prose.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-322

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-322)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,7 +1,5 @@
 {{% notice
-message="quoted %}} keeps   its spacing"
-%}}
-Format this paragraph as Markdown.
+message="quoted %}} keeps its spacing" %}} Format this paragraph as Markdown.
 
 # A heading
 
```

```json
{
  "stdin": "{{% notice\r\nmessage=\"quoted %}} keeps   its spacing\"\r\n%}}\r\nFormat this paragraph as Markdown.\r\n\r\n# A heading\r\n\r\n{{% unrelated %}}\r\nFollowing prose.\r\n",
  "reviewed_stdout": "{{% notice\r\nmessage=\"quoted %}} keeps   its spacing\"\r\n%}}\r\nFormat this paragraph as Markdown.\r\n\r\n# A heading\r\n\r\n{{% unrelated %}}\r\nFollowing prose.\r\n",
  "current_stdout": "{{% notice\r\nmessage=\"quoted %}} keeps its spacing\" %}} Format this paragraph as Markdown.\r\n\r\n# A heading\r\n\r\n{{% unrelated %}}\r\nFollowing prose.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-323

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-323)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,11 +1,12 @@
 {{< notice
-message=`keep   this  
+message=`keep   this
 
 and   this`
 >}}
-Format this paragraph as Markdown.
+Format   this paragraph
+as Markdown.
 
-# A heading
+#   A heading
 
 {{< /notice >}}
 Following prose.
```

```json
{
  "stdin": "{{< notice\r\nmessage=`keep   this  \r\n\r\nand   this`\r\n>}}\r\nFormat   this paragraph\r\nas Markdown.\r\n\r\n#   A heading\r\n\r\n{{< /notice >}}\r\nFollowing\r\nprose.\r\n",
  "reviewed_stdout": "{{< notice\r\nmessage=`keep   this  \r\n\r\nand   this`\r\n>}}\r\nFormat this paragraph as Markdown.\r\n\r\n# A heading\r\n\r\n{{< /notice >}}\r\nFollowing prose.\r\n",
  "current_stdout": "{{< notice\r\nmessage=`keep   this\r\n\r\nand   this`\r\n>}}\r\nFormat   this paragraph\r\nas Markdown.\r\n\r\n#   A heading\r\n\r\n{{< /notice >}}\r\nFollowing prose.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-324

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-324)

```text
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,5 @@
 {{< notice
-message=`keep   this  
+message=`keep   this
 
 and   this`
 >}}
```

```json
{
  "stdin": "{{< notice\r\nmessage=`keep   this  \r\n\r\nand   this`\r\n>}}\r\nFormat this paragraph as Markdown.\r\n\r\n# A heading\r\n\r\n{{< /notice >}}\r\nFollowing prose.\r\n",
  "reviewed_stdout": "{{< notice\r\nmessage=`keep   this  \r\n\r\nand   this`\r\n>}}\r\nFormat this paragraph as Markdown.\r\n\r\n# A heading\r\n\r\n{{< /notice >}}\r\nFollowing prose.\r\n",
  "current_stdout": "{{< notice\r\nmessage=`keep   this\r\n\r\nand   this`\r\n>}}\r\nFormat this paragraph as Markdown.\r\n\r\n# A heading\r\n\r\n{{< /notice >}}\r\nFollowing prose.\r\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-355

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-355) · [active transcript](../../cases/recovered_pr4_template_delimiters_inside_complete_code_spans_355.case) · [reviewed transcript](recovered_pr4_template_delimiters_inside_complete_code_spans_355.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with `{{ keep `` this }}`.
-Second sentence.
+First
+sentence with `{{ keep `` this }}`. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with `{{ keep `` this }}`. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with `{{ keep `` this }}`.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with `{{ keep `` this }}`. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-357

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-357) · [active transcript](../../cases/recovered_pr4_template_delimiters_inside_complete_code_spans_357.case) · [reviewed transcript](recovered_pr4_template_delimiters_inside_complete_code_spans_357.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with ``{{ keep ``` this }}``.
-Second sentence.
+First
+sentence with ``{{ keep ``` this }}``. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with ``{{ keep ``` this }}``. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with ``{{ keep ``` this }}``.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with ``{{ keep ``` this }}``. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-359

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-359) · [active transcript](../../cases/recovered_pr4_template_delimiters_inside_complete_code_spans_359.case) · [reviewed transcript](recovered_pr4_template_delimiters_inside_complete_code_spans_359.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with ``{{ keep ``` x ` _this_ }}``.
-Second sentence.
+First
+sentence with ``{{ keep ``` x ` _this_ }}``. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with ``{{ keep ``` x ` _this_ }}``. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with ``{{ keep ``` x ` _this_ }}``.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with ``{{ keep ``` x ` _this_ }}``. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-361

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-361) · [active transcript](../../cases/recovered_pr4_template_delimiters_inside_complete_code_spans_361.case) · [reviewed transcript](recovered_pr4_template_delimiters_inside_complete_code_spans_361.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,4 @@
-First sentence with `{{ keep
-this }}`.
-Second sentence.
+First
+sentence with `{{ keep
+this }}`. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with `{{ keep\nthis }}`. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with `{{ keep\nthis }}`.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with `{{ keep\nthis }}`. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-369

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-369) · [active transcript](../../cases/recovered_pr4_template_delimiters_inside_complete_code_spans_369.case) · [reviewed transcript](recovered_pr4_template_delimiters_inside_complete_code_spans_369.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with ``{{< note text="}}` keep   _this_" >}}``.
-Second sentence.
+First
+sentence with ``{{< note text="}}` keep   _this_" >}}``. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with ``{{< note text=\"}}` keep   _this_\" >}}``. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with ``{{< note text=\"}}` keep   _this_\" >}}``.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with ``{{< note text=\"}}` keep   _this_\" >}}``. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-371

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-371) · [active transcript](../../cases/recovered_pr4_template_delimiters_inside_complete_code_spans_371.case) · [reviewed transcript](recovered_pr4_template_delimiters_inside_complete_code_spans_371.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with `{{< note text="}} keep   _this_" >}}`.
-Second sentence.
+First
+sentence with `{{< note text="}} keep   _this_" >}}`. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with `{{< note text=\"}} keep   _this_\" >}}`. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with `{{< note text=\"}} keep   _this_\" >}}`.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with `{{< note text=\"}} keep   _this_\" >}}`. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-373

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-373) · [active transcript](../../cases/recovered_pr4_wrap_around_protected_template_tokens_373.case) · [reviewed transcript](recovered_pr4_wrap_around_protected_template_tokens_373.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with {{ keep   this }}.
-Second sentence.
+First
+sentence with {{ keep   this }}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with {{ keep   this }}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with {{ keep   this }}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with {{ keep   this }}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-375

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-375) · [active transcript](../../cases/recovered_pr4_wrap_around_protected_template_tokens_375.case) · [reviewed transcript](recovered_pr4_wrap_around_protected_template_tokens_375.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with `code` {{ keep   this }}.
-Second sentence.
+First
+sentence with `code` {{ keep   this }}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with `code` {{ keep   this }}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with `code` {{ keep   this }}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with `code` {{ keep   this }}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-377

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-377) · [active transcript](../../cases/recovered_pr4_wrap_around_protected_template_tokens_377.case) · [reviewed transcript](recovered_pr4_wrap_around_protected_template_tokens_377.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with {{ keep   this }} `code`.
-Second sentence.
+First
+sentence with {{ keep   this }} `code`. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with {{ keep   this }} `code`. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with {{ keep   this }} `code`.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with {{ keep   this }} `code`. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-379

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-379) · [active transcript](../../cases/recovered_pr4_wrap_around_protected_template_tokens_379.case) · [reviewed transcript](recovered_pr4_wrap_around_protected_template_tokens_379.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with `{{ code }}` then {{ keep   this }}.
-Second sentence.
+First
+sentence with `{{ code }}` then {{ keep   this }}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with `{{ code }}` then {{ keep   this }}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with `{{ code }}` then {{ keep   this }}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with `{{ code }}` then {{ keep   this }}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-381

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-381) · [active transcript](../../cases/recovered_pr4_wrap_around_protected_template_tokens_381.case) · [reviewed transcript](recovered_pr4_wrap_around_protected_template_tokens_381.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with `{{ code }}` then {% keep   this %}.
-Second sentence.
+First
+sentence with `{{ code }}` then {% keep   this %}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with `{{ code }}` then {% keep   this %}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with `{{ code }}` then {% keep   this %}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with `{{ code }}` then {% keep   this %}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-383

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-383) · [active transcript](../../cases/recovered_pr4_wrap_around_protected_template_tokens_383.case) · [reviewed transcript](recovered_pr4_wrap_around_protected_template_tokens_383.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with {{ keep `code`   this }}.
-Second sentence.
+First
+sentence with {{ keep `code`   this }}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with {{ keep `code`   this }}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with {{ keep `code`   this }}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with {{ keep `code`   this }}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-385

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-385) · [active transcript](../../cases/recovered_pr4_wrap_around_protected_template_tokens_385.case) · [reviewed transcript](recovered_pr4_wrap_around_protected_template_tokens_385.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with {{ keep   _this_ }}.
-Second sentence.
+First
+sentence with {{ keep   _this_ }}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with {{ keep   _this_ }}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with {{ keep   _this_ }}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with {{ keep   _this_ }}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-387

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-387) · [active transcript](../../cases/recovered_pr4_wrap_around_protected_template_tokens_387.case) · [reviewed transcript](recovered_pr4_wrap_around_protected_template_tokens_387.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with {% keep   this %}.
-Second sentence.
+First
+sentence with {% keep   this %}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with {% keep   this %}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with {% keep   this %}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with {% keep   this %}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-389

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-389) · [active transcript](../../cases/recovered_pr4_wrap_around_protected_template_tokens_389.case) · [reviewed transcript](recovered_pr4_wrap_around_protected_template_tokens_389.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with {# keep   this #}.
-Second sentence.
+First
+sentence with {# keep   this #}. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with {# keep   this #}. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with {# keep   this #}.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with {# keep   this #}. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-391

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-391) · [active transcript](../../cases/recovered_pr4_wrap_around_protected_template_tokens_391.case) · [reviewed transcript](recovered_pr4_wrap_around_protected_template_tokens_391.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with [target](https://example.com/`{{key}}`).
-Second sentence.
+First
+sentence with [target](https://example.com/`{{key}}`). Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with [target](https://example.com/`{{key}}`). Second\nsentence.\n",
  "reviewed_stdout": "First sentence with [target](https://example.com/`{{key}}`).\nSecond sentence.\n",
  "current_stdout": "First\nsentence with [target](https://example.com/`{{key}}`). Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-393

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-393) · [active transcript](../../cases/recovered_pr4_wrap_around_protected_template_tokens_393.case) · [reviewed transcript](recovered_pr4_wrap_around_protected_template_tokens_393.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with [target](url "`{{ title }}`").
-Second sentence.
+First
+sentence with [target](url "`{{ title }}`"). Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with [target](url \"`{{ title }}`\"). Second\nsentence.\n",
  "reviewed_stdout": "First sentence with [target](url \"`{{ title }}`\").\nSecond sentence.\n",
  "current_stdout": "First\nsentence with [target](url \"`{{ title }}`\"). Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-395

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-395) · [active transcript](../../cases/recovered_pr4_wrap_around_protected_template_tokens_395.case) · [reviewed transcript](recovered_pr4_wrap_around_protected_template_tokens_395.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with [![alt](inner)](outer "`{{ title }}`").
-Second sentence.
+First
+sentence with [![alt](inner)](outer "`{{ title }}`"). Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with [![alt](inner)](outer \"`{{ title }}`\"). Second\nsentence.\n",
  "reviewed_stdout": "First sentence with [![alt](inner)](outer \"`{{ title }}`\").\nSecond sentence.\n",
  "current_stdout": "First\nsentence with [![alt](inner)](outer \"`{{ title }}`\"). Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-397

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-397) · [active transcript](../../cases/recovered_pr4_wrap_around_protected_template_tokens_397.case) · [reviewed transcript](recovered_pr4_wrap_around_protected_template_tokens_397.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
format --canonical --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with $`{{ math }}`$.
-Second sentence.
+First
+sentence with $`{{ math }}`$. Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with $`{{ math }}`$. Second\nsentence.\n",
  "reviewed_stdout": "First sentence with $`{{ math }}`$.\nSecond sentence.\n",
  "current_stdout": "First\nsentence with $`{{ math }}`$. Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-409

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-409) · [active transcript](../../cases/recovered_pr4_inline_code_templates_in_other_markdown_blocks_409.case) · [reviewed transcript](recovered_pr4_inline_code_templates_in_other_markdown_blocks_409.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1 @@
-# `{{ title }}`
+#   `{{ title }}` ##
```

```json
{
  "stdin": "#   `{{ title }}` ##\n",
  "reviewed_stdout": "# `{{ title }}`\n",
  "current_stdout": "#   `{{ title }}` ##\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-411

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-411) · [active transcript](../../cases/recovered_pr4_inline_code_templates_in_other_markdown_blocks_411.case) · [reviewed transcript](recovered_pr4_inline_code_templates_in_other_markdown_blocks_411.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1,2 @@
-# `{{ title }}`
+`{{ title }}`
+====
```

```json
{
  "stdin": "`{{ title }}`\n====\n",
  "reviewed_stdout": "# `{{ title }}`\n",
  "current_stdout": "`{{ title }}`\n====\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-412

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-412) · [active transcript](../../cases/recovered_pr4_inline_code_templates_in_other_markdown_blocks_412.case) · [reviewed transcript](recovered_pr4_inline_code_templates_in_other_markdown_blocks_412.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-- First sentence with `{{ foo }}`.
-  Second sentence.
+- First
+  sentence with `{{ foo }}`. Second
+  sentence.
```

```json
{
  "stdin": "- First\n  sentence with `{{ foo }}`. Second\n  sentence.\n",
  "reviewed_stdout": "- First sentence with `{{ foo }}`.\n  Second sentence.\n",
  "current_stdout": "- First\n  sentence with `{{ foo }}`. Second\n  sentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-414

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-414) · [active transcript](../../cases/recovered_pr4_inline_code_templates_in_other_markdown_blocks_414.case) · [reviewed transcript](recovered_pr4_inline_code_templates_in_other_markdown_blocks_414.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-> First sentence with `{{ foo }}`.
-> Second sentence.
+> First
+> sentence with `{{ foo }}`. Second
+> sentence.
```

```json
{
  "stdin": "> First\n> sentence with `{{ foo }}`. Second\n> sentence.\n",
  "reviewed_stdout": "> First sentence with `{{ foo }}`.\n> Second sentence.\n",
  "current_stdout": "> First\n> sentence with `{{ foo }}`. Second\n> sentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-416

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-416) · [active transcript](../../cases/recovered_pr4_inline_code_templates_in_other_markdown_blocks_416.case) · [reviewed transcript](recovered_pr4_inline_code_templates_in_other_markdown_blocks_416.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,2 +1,3 @@
-First sentence with [`{{ foo }}`](url).
-Second sentence.
+First
+sentence with [`{{ foo }}`](url). Second
+sentence.
```

```json
{
  "stdin": "First\nsentence with [`{{ foo }}`](url). Second\nsentence.\n",
  "reviewed_stdout": "First sentence with [`{{ foo }}`](url).\nSecond sentence.\n",
  "current_stdout": "First\nsentence with [`{{ foo }}`](url). Second\nsentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-418

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-418) · [active transcript](../../cases/recovered_pr4_nested_inline_code_templates_allow_wrapping_418.case) · [reviewed transcript](recovered_pr4_nested_inline_code_templates_allow_wrapping_418.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1,2 @@
-Before *`<% keep   this %>`* after.
+Before
+*`<% keep   this %>`* after.
```

```json
{
  "stdin": "Before\n*`<% keep   this %>`* after.\n",
  "reviewed_stdout": "Before *`<% keep   this %>`* after.\n",
  "current_stdout": "Before\n*`<% keep   this %>`* after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-420

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-420) · [active transcript](../../cases/recovered_pr4_nested_inline_code_templates_allow_wrapping_420.case) · [reviewed transcript](recovered_pr4_nested_inline_code_templates_allow_wrapping_420.case)

```text
format --wrap sentence --stdin-file-path input.md --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1 +1,2 @@
-Before [`<% keep   this %>`](url) after.
+Before
+[`<% keep   this %>`](url) after.
```

```json
{
  "stdin": "Before\n[`<% keep   this %>`](url) after.\n",
  "reviewed_stdout": "Before [`<% keep   this %>`](url) after.\n",
  "current_stdout": "Before\n[`<% keep   this %>`](url) after.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-426

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-426) · [active transcript](../../cases/recovered_pr4_inline_code_templates_in_marked_markdown_426.case) · [reviewed transcript](recovered_pr4_inline_code_templates_in_marked_markdown_426.case)

```text
format --wrap sentence --stdin-file-path input.py --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,6 @@
 # fmt: markdown
 text = """
-First sentence with `{{ foo }}`.
-Second sentence.
+First
+sentence with `{{ foo }}`. Second
+sentence.
 """
```

```json
{
  "stdin": "# fmt: markdown\ntext = \"\"\"\nFirst\nsentence with `{{ foo }}`. Second\nsentence.\n\"\"\"\n",
  "reviewed_stdout": "# fmt: markdown\ntext = \"\"\"\nFirst sentence with `{{ foo }}`.\nSecond sentence.\n\"\"\"\n",
  "current_stdout": "# fmt: markdown\ntext = \"\"\"\nFirst\nsentence with `{{ foo }}`. Second\nsentence.\n\"\"\"\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-428

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-428) · [active transcript](../../cases/recovered_pr4_inline_code_templates_in_marked_markdown_428.case) · [reviewed transcript](recovered_pr4_inline_code_templates_in_marked_markdown_428.case)

```text
format --wrap sentence --stdin-file-path input.R --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,6 @@
 # fmt: markdown
 text <- r"(
-First sentence with `{{ foo }}`.
-Second sentence.
+First
+sentence with `{{ foo }}`. Second
+sentence.
 )"
```

```json
{
  "stdin": "# fmt: markdown\ntext <- r\"(\nFirst\nsentence with `{{ foo }}`. Second\nsentence.\n)\"\n",
  "reviewed_stdout": "# fmt: markdown\ntext <- r\"(\nFirst sentence with `{{ foo }}`.\nSecond sentence.\n)\"\n",
  "current_stdout": "# fmt: markdown\ntext <- r\"(\nFirst\nsentence with `{{ foo }}`. Second\nsentence.\n)\"\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-430

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-430) · [active transcript](../../cases/recovered_pr4_inline_code_templates_in_marked_markdown_430.case) · [reviewed transcript](recovered_pr4_inline_code_templates_in_marked_markdown_430.case)

```text
format --wrap sentence --stdin-file-path input.yaml --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,4 @@
 text: !markdown |
-  First sentence with `{{ foo }}`.
-  Second sentence.
+  First
+  sentence with `{{ foo }}`. Second
+  sentence.
```

```json
{
  "stdin": "text: !markdown |\n  First\n  sentence with `{{ foo }}`. Second\n  sentence.\n",
  "reviewed_stdout": "text: !markdown |\n  First sentence with `{{ foo }}`.\n  Second sentence.\n",
  "current_stdout": "text: !markdown |\n  First\n  sentence with `{{ foo }}`. Second\n  sentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-432

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-432) · [active transcript](../../cases/recovered_pr4_inline_code_templates_in_marked_markdown_432.case) · [reviewed transcript](recovered_pr4_inline_code_templates_in_marked_markdown_432.case)

```text
format --wrap sentence --stdin-file-path input.yaml --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,5 @@
 # fmt: markdown
 text: |
-  First sentence with `{{ foo }}`.
-  Second sentence.
+  First
+  sentence with `{{ foo }}`. Second
+  sentence.
```

```json
{
  "stdin": "# fmt: markdown\ntext: |\n  First\n  sentence with `{{ foo }}`. Second\n  sentence.\n",
  "reviewed_stdout": "# fmt: markdown\ntext: |\n  First sentence with `{{ foo }}`.\n  Second sentence.\n",
  "current_stdout": "# fmt: markdown\ntext: |\n  First\n  sentence with `{{ foo }}`. Second\n  sentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-438

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-438) · [active transcript](../../cases/recovered_pr4_inline_code_templates_in_marked_markdown_438.case) · [reviewed transcript](recovered_pr4_inline_code_templates_in_marked_markdown_438.case)

```text
format --wrap sentence --stdin-file-path input.py --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,6 @@
 # fmt: markdown
 text = """
-First sentence with {{ foo }}.
-Second sentence.
+First
+sentence with {{ foo }}. Second
+sentence.
 """
```

```json
{
  "stdin": "# fmt: markdown\ntext = \"\"\"\nFirst\nsentence with {{ foo }}. Second\nsentence.\n\"\"\"\n",
  "reviewed_stdout": "# fmt: markdown\ntext = \"\"\"\nFirst sentence with {{ foo }}.\nSecond sentence.\n\"\"\"\n",
  "current_stdout": "# fmt: markdown\ntext = \"\"\"\nFirst\nsentence with {{ foo }}. Second\nsentence.\n\"\"\"\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-440

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-440) · [active transcript](../../cases/recovered_pr4_inline_code_templates_in_marked_markdown_440.case) · [reviewed transcript](recovered_pr4_inline_code_templates_in_marked_markdown_440.case)

```text
format --wrap sentence --stdin-file-path input.R --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,6 @@
 # fmt: markdown
 text <- r"(
-First sentence with {{ foo }}.
-Second sentence.
+First
+sentence with {{ foo }}. Second
+sentence.
 )"
```

```json
{
  "stdin": "# fmt: markdown\ntext <- r\"(\nFirst\nsentence with {{ foo }}. Second\nsentence.\n)\"\n",
  "reviewed_stdout": "# fmt: markdown\ntext <- r\"(\nFirst sentence with {{ foo }}.\nSecond sentence.\n)\"\n",
  "current_stdout": "# fmt: markdown\ntext <- r\"(\nFirst\nsentence with {{ foo }}. Second\nsentence.\n)\"\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-442

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-442) · [active transcript](../../cases/recovered_pr4_inline_code_templates_in_marked_markdown_442.case) · [reviewed transcript](recovered_pr4_inline_code_templates_in_marked_markdown_442.case)

```text
format --wrap sentence --stdin-file-path input.yaml --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,4 @@
 text: !markdown |
-  First sentence with {{ foo }}.
-  Second sentence.
+  First
+  sentence with {{ foo }}. Second
+  sentence.
```

```json
{
  "stdin": "text: !markdown |\n  First\n  sentence with {{ foo }}. Second\n  sentence.\n",
  "reviewed_stdout": "text: !markdown |\n  First sentence with {{ foo }}.\n  Second sentence.\n",
  "current_stdout": "text: !markdown |\n  First\n  sentence with {{ foo }}. Second\n  sentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-444

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-444) · [active transcript](../../cases/recovered_pr4_inline_code_templates_in_marked_markdown_444.case) · [reviewed transcript](recovered_pr4_inline_code_templates_in_marked_markdown_444.case)

```text
format --wrap sentence --stdin-file-path input.yaml --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,5 @@
 # fmt: markdown
 text: |
-  First sentence with {{ foo }}.
-  Second sentence.
+  First
+  sentence with {{ foo }}. Second
+  sentence.
```

```json
{
  "stdin": "# fmt: markdown\ntext: |\n  First\n  sentence with {{ foo }}. Second\n  sentence.\n",
  "reviewed_stdout": "# fmt: markdown\ntext: |\n  First sentence with {{ foo }}.\n  Second sentence.\n",
  "current_stdout": "# fmt: markdown\ntext: |\n  First\n  sentence with {{ foo }}. Second\n  sentence.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-448

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-448) · [active transcript](../../cases/recovered_pr4_markdown_div_shortcode_math.case) · [reviewed transcript](recovered_pr4_markdown_div_shortcode_math.case)

```text
format --stdin-file-path input.md --wrap 40
```

```diff
--- reviewed stdout
+++ main stdout
@@ -38,7 +38,7 @@
 after an opaque shortcode block.
 
 {{% notice %}}
-Format this body as Markdown.
+Format   this body as Markdown.
 {{% /notice %}}
 
 $$
```

```json
{
  "stdin": "::: {.callout-tip}\n## Learn more\n{{< meta title >}}\n\n```{=html}\n<div class=\"raw\">HTML</div>\n```\n\nThis paragraph is intentionally long enough to be considered for wrapping by the markdown formatter inside a fenced div.\n:::\n\n:::: columns\n::: {.column width=\"50%\"}\n| a | b |\n|---|---|\n| 1 | 2 |\n:::\n::: {.column width=\"50%\"}\nThis paragraph is intentionally long enough to be considered for wrapping by the markdown formatter inside a nested div.\n:::\n::::\n\nThis paragraph is intentionally long enough to be considered for wrapping before an opaque shortcode block.\n\n{{< meta title >}}\n\nThis paragraph is intentionally long enough to be considered for wrapping after an opaque shortcode block.\n\n{{% notice %}}\nFormat   this body as Markdown.\n{{% /notice %}}\n\n$$\n\\begin{aligned}\na^2 + b^2 &= c^2\n\\end{aligned}\n$$\n\n\\[\nE = mc^2\n\\]\n\n\\begin{equation}\nx = y\n\\end{equation}\n\n- Use the identity:\n\n  $$\n  x^2 + 2x + 1 = (x + 1)^2\n  $$\n",
  "reviewed_stdout": "::: {.callout-tip}\n## Learn more\n{{< meta title >}}\n\n```{=html}\n<div class=\"raw\">HTML</div>\n```\n\nThis paragraph is intentionally long\nenough to be considered for wrapping by\nthe markdown formatter inside a fenced\ndiv.\n:::\n\n:::: columns\n::: {.column width=\"50%\"}\n| a   | b   |\n| --- | --- |\n| 1   | 2   |\n:::\n\n::: {.column width=\"50%\"}\nThis paragraph is intentionally long\nenough to be considered for wrapping by\nthe markdown formatter inside a nested\ndiv.\n:::\n::::\n\nThis paragraph is intentionally long\nenough to be considered for wrapping\nbefore an opaque shortcode block.\n\n{{< meta title >}}\n\nThis paragraph is intentionally long\nenough to be considered for wrapping\nafter an opaque shortcode block.\n\n{{% notice %}}\nFormat this body as Markdown.\n{{% /notice %}}\n\n$$\n\\begin{aligned}\na^2 + b^2 &= c^2\n\\end{aligned}\n$$\n\n\\[\nE = mc^2\n\\]\n\n\\begin{equation}\nx = y\n\\end{equation}\n\n- Use the identity:\n\n  $$\n  x^2 + 2x + 1 = (x + 1)^2\n  $$\n",
  "current_stdout": "::: {.callout-tip}\n## Learn more\n{{< meta title >}}\n\n```{=html}\n<div class=\"raw\">HTML</div>\n```\n\nThis paragraph is intentionally long\nenough to be considered for wrapping by\nthe markdown formatter inside a fenced\ndiv.\n:::\n\n:::: columns\n::: {.column width=\"50%\"}\n| a   | b   |\n| --- | --- |\n| 1   | 2   |\n:::\n\n::: {.column width=\"50%\"}\nThis paragraph is intentionally long\nenough to be considered for wrapping by\nthe markdown formatter inside a nested\ndiv.\n:::\n::::\n\nThis paragraph is intentionally long\nenough to be considered for wrapping\nbefore an opaque shortcode block.\n\n{{< meta title >}}\n\nThis paragraph is intentionally long\nenough to be considered for wrapping\nafter an opaque shortcode block.\n\n{{% notice %}}\nFormat   this body as Markdown.\n{{% /notice %}}\n\n$$\n\\begin{aligned}\na^2 + b^2 &= c^2\n\\end{aligned}\n$$\n\n\\[\nE = mc^2\n\\]\n\n\\begin{equation}\nx = y\n\\end{equation}\n\n- Use the identity:\n\n  $$\n  x^2 + 2x + 1 = (x + 1)^2\n  $$\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-449

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-449) · [active transcript](../../cases/recovered_pr4_markdown_inline_code_template_sentence_wrap.case) · [reviewed transcript](recovered_pr4_markdown_inline_code_template_sentence_wrap.case)

```text
format --stdin-file-path input.md --wrap sentence --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -7,8 +7,12 @@
 Fifth sentence with ``{{ render("`code`", {"label": "keep   this"}) }}``.
 Sixth sentence.
 
-Seventh sentence with {{ render({"label": "keep   this"}) }}.
-Eighth sentence.
+Seventh
+sentence with {{ render({"label": "keep   this"}) }}. Eighth
+sentence.
 
-The delivery instructions for this account use {{ render_customer_notice( customer.account_name,  customer.preferred_delivery_address ) }} before shipping.
-Confirmation follows.
+The delivery
+instructions for this account use
+{{ render_customer_notice( customer.account_name,  customer.preferred_delivery_address ) }} before
+shipping. Confirmation
+follows.
```

```json
{
  "stdin": "First\nsentence with `${{ foo }}`. Second\nsentence.\n\nThird\nsentence with {{ foo }}. Fourth\nsentence.\n\nFifth\nsentence with ``{{ render(\"`code`\", {\"label\": \"keep   this\"}) }}``. Sixth\nsentence.\n\nSeventh\nsentence with {{ render({\"label\": \"keep   this\"}) }}. Eighth\nsentence.\n\nThe delivery\ninstructions for this account use\n{{ render_customer_notice( customer.account_name,  customer.preferred_delivery_address ) }} before\nshipping. Confirmation\nfollows.\n",
  "reviewed_stdout": "First sentence with `${{ foo }}`.\nSecond sentence.\n\nThird sentence with {{ foo }}.\nFourth sentence.\n\nFifth sentence with ``{{ render(\"`code`\", {\"label\": \"keep   this\"}) }}``.\nSixth sentence.\n\nSeventh sentence with {{ render({\"label\": \"keep   this\"}) }}.\nEighth sentence.\n\nThe delivery instructions for this account use {{ render_customer_notice( customer.account_name,  customer.preferred_delivery_address ) }} before shipping.\nConfirmation follows.\n",
  "current_stdout": "First sentence with `${{ foo }}`.\nSecond sentence.\n\nThird sentence with {{ foo }}.\nFourth sentence.\n\nFifth sentence with ``{{ render(\"`code`\", {\"label\": \"keep   this\"}) }}``.\nSixth sentence.\n\nSeventh\nsentence with {{ render({\"label\": \"keep   this\"}) }}. Eighth\nsentence.\n\nThe delivery\ninstructions for this account use\n{{ render_customer_notice( customer.account_name,  customer.preferred_delivery_address ) }} before\nshipping. Confirmation\nfollows.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-450

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-450) · [active transcript](../../cases/recovered_pr4_markdown_template_blocks_column_wrap.case) · [reviewed transcript](recovered_pr4_markdown_template_blocks_column_wrap.case)

```text
format --stdin-file-path input.md --wrap 40 --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -9,12 +9,9 @@
 wrap across several lines.
 
 {{% notice title="An overwide template block title that must remain exactly as authored" %}}
-This paragraph between the tags has
-irregular spacing and enough ordinary
-prose to wrap at the requested column.
+This paragraph between the tags has irregular   spacing and enough ordinary prose to wrap at the requested column.
 
-A second paragraph between the tags also
-wraps as ordinary Markdown.
+A second   paragraph between the tags also wraps as ordinary Markdown.
 {{% /notice %}}
 
 {{< highlight go >}}
```

```json
{
  "stdin": "This paragraph before the template blocks is long enough to wrap at the requested column.\n\n{{< figure src=\"a-long-image-name.png\" caption=\"An overwide caption that must stay exactly as authored\" >}}\n\nThe paragraph between these template blocks also has enough ordinary prose to wrap across several lines.\n\n{{% notice title=\"An overwide template block title that must remain exactly as authored\" %}}\nThis paragraph between the tags has irregular   spacing and enough ordinary prose to wrap at the requested column.\n\nA second   paragraph between the tags also wraps as ordinary Markdown.\n{{% /notice %}}\n\n{{< highlight go >}}\n```go\nfunc main() {\n    fmt.Println(\"This overwide string is preserved by the Markdown code fence.\")\n}\n```\n{{< /highlight >}}\n\nThis paragraph after the template region is long enough to wrap at the requested column.\n",
  "reviewed_stdout": "This paragraph before the template\nblocks is long enough to wrap at the\nrequested column.\n\n{{< figure src=\"a-long-image-name.png\" caption=\"An overwide caption that must stay exactly as authored\" >}}\n\nThe paragraph between these template\nblocks also has enough ordinary prose to\nwrap across several lines.\n\n{{% notice title=\"An overwide template block title that must remain exactly as authored\" %}}\nThis paragraph between the tags has\nirregular spacing and enough ordinary\nprose to wrap at the requested column.\n\nA second paragraph between the tags also\nwraps as ordinary Markdown.\n{{% /notice %}}\n\n{{< highlight go >}}\n```go\nfunc main() {\n    fmt.Println(\"This overwide string is preserved by the Markdown code fence.\")\n}\n```\n{{< /highlight >}}\n\nThis paragraph after the template region\nis long enough to wrap at the requested\ncolumn.\n",
  "current_stdout": "This paragraph before the template\nblocks is long enough to wrap at the\nrequested column.\n\n{{< figure src=\"a-long-image-name.png\" caption=\"An overwide caption that must stay exactly as authored\" >}}\n\nThe paragraph between these template\nblocks also has enough ordinary prose to\nwrap across several lines.\n\n{{% notice title=\"An overwide template block title that must remain exactly as authored\" %}}\nThis paragraph between the tags has irregular   spacing and enough ordinary prose to wrap at the requested column.\n\nA second   paragraph between the tags also wraps as ordinary Markdown.\n{{% /notice %}}\n\n{{< highlight go >}}\n```go\nfunc main() {\n    fmt.Println(\"This overwide string is preserved by the Markdown code fence.\")\n}\n```\n{{< /highlight >}}\n\nThis paragraph after the template region\nis long enough to wrap at the requested\ncolumn.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-451

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-451) · [active transcript](../../cases/recovered_pr4_markdown_template_code_boundaries.case) · [reviewed transcript](recovered_pr4_markdown_template_code_boundaries.case)

```text
format --stdin-file-path input.md --wrap sentence --canonical --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -6,7 +6,9 @@
 `{{% note text='}}` keep   _this_' %}}
 after.
 
-Before ``{{< note text="}}` keep   _this_" >}}`` after.
+Before
+``{{< note text="}}` keep   _this_" >}}``
+after.
 
 Before <span><script>const x = "keep   this"; {{ foo }}</span>
 
```

```json
{
  "stdin": "Before\n`{{< note text=\"}}` keep   _this_\" >}}\nafter.\n\nBefore\n`{{% note text='}}` keep   _this_' %}}\nafter.\n\nBefore\n``{{< note text=\"}}` keep   _this_\" >}}``\nafter.\n\nBefore <span><script>const x = \"keep   this\"; {{ foo }}</span>\n\nBefore ${{ \"keep $   this\" }} after.\n\n<span>{ <span> } </span> keep   _this_ {{ foo }}</span>\n\nFollowing\n_prose_.\n",
  "reviewed_stdout": "Before\n`{{< note text=\"}}` keep   _this_\" >}}\nafter.\n\nBefore\n`{{% note text='}}` keep   _this_' %}}\nafter.\n\nBefore ``{{< note text=\"}}` keep   _this_\" >}}`` after.\n\nBefore <span><script>const x = \"keep   this\"; {{ foo }}</span>\n\nBefore ${{ \"keep $   this\" }} after.\n\n<span>{ <span> } </span> keep   _this_ {{ foo }}</span>\n\nFollowing *prose*.\n",
  "current_stdout": "Before\n`{{< note text=\"}}` keep   _this_\" >}}\nafter.\n\nBefore\n`{{% note text='}}` keep   _this_' %}}\nafter.\n\nBefore\n``{{< note text=\"}}` keep   _this_\" >}}``\nafter.\n\nBefore <span><script>const x = \"keep   this\"; {{ foo }}</span>\n\nBefore ${{ \"keep $   this\" }} after.\n\n<span>{ <span> } </span> keep   _this_ {{ foo }}</span>\n\nFollowing *prose*.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-452

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-452) · [active transcript](../../cases/recovered_pr4_markdown_template_column_wrap.case) · [reviewed transcript](recovered_pr4_markdown_template_column_wrap.case)

```text
format --stdin-file-path input.md --wrap 40 --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -3,10 +3,11 @@
 across several lines without splitting
 either expression.
 
-The delivery instructions for this
-account use
-{{ render_customer_notice( customer.account_name,  customer.preferred_delivery_address ) }}
-before shipping. Confirmation follows.
+The delivery
+instructions for this account use
+{{ render_customer_notice( customer.account_name,  customer.preferred_delivery_address ) }} before
+shipping. Confirmation
+follows.
 
 The delivery instructions for this
 account use
@@ -17,8 +18,4 @@
 {{ customer.delivery_address }} and the
 confirmation follows.
 
-Before
-{% if customer.account_is_active and customer.preferred_delivery_address_is_verified %}
-the ordinary prose in this conditional
-still wraps at the requested column
-{% endif %} and more prose follows.
+Before {% if customer.account_is_active and customer.preferred_delivery_address_is_verified %} the ordinary prose in this conditional still wraps at the requested column {% endif %} and more prose follows.
```

```json
{
  "stdin": "This paragraph uses {{ foo }} beside `${{ bar }}` so ordinary prose wraps across several lines without splitting either expression.\n\nThe delivery\ninstructions for this account use\n{{ render_customer_notice( customer.account_name,  customer.preferred_delivery_address ) }} before\nshipping. Confirmation\nfollows.\n\nThe delivery\ninstructions for this account use\n`{{ render_customer_notice( customer.account_name,  customer.preferred_delivery_address ) }}` before\nshipping. Confirmation\nfollows.\n\nThe delivery address is {{ customer.delivery_address }} and the confirmation follows.\n\nBefore {% if customer.account_is_active and customer.preferred_delivery_address_is_verified %} the ordinary prose in this conditional still wraps at the requested column {% endif %} and more prose follows.\n",
  "reviewed_stdout": "This paragraph uses {{ foo }} beside\n`${{ bar }}` so ordinary prose wraps\nacross several lines without splitting\neither expression.\n\nThe delivery instructions for this\naccount use\n{{ render_customer_notice( customer.account_name,  customer.preferred_delivery_address ) }}\nbefore shipping. Confirmation follows.\n\nThe delivery instructions for this\naccount use\n`{{ render_customer_notice( customer.account_name,  customer.preferred_delivery_address ) }}`\nbefore shipping. Confirmation follows.\n\nThe delivery address is\n{{ customer.delivery_address }} and the\nconfirmation follows.\n\nBefore\n{% if customer.account_is_active and customer.preferred_delivery_address_is_verified %}\nthe ordinary prose in this conditional\nstill wraps at the requested column\n{% endif %} and more prose follows.\n",
  "current_stdout": "This paragraph uses {{ foo }} beside\n`${{ bar }}` so ordinary prose wraps\nacross several lines without splitting\neither expression.\n\nThe delivery\ninstructions for this account use\n{{ render_customer_notice( customer.account_name,  customer.preferred_delivery_address ) }} before\nshipping. Confirmation\nfollows.\n\nThe delivery instructions for this\naccount use\n`{{ render_customer_notice( customer.account_name,  customer.preferred_delivery_address ) }}`\nbefore shipping. Confirmation follows.\n\nThe delivery address is\n{{ customer.delivery_address }} and the\nconfirmation follows.\n\nBefore {% if customer.account_is_active and customer.preferred_delivery_address_is_verified %} the ordinary prose in this conditional still wraps at the requested column {% endif %} and more prose follows.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-453

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-453) · [active transcript](../../cases/recovered_pr4_markdown_template_multiline_wrap.case) · [reviewed transcript](recovered_pr4_markdown_template_multiline_wrap.case)

```text
format --stdin-file-path input.md --wrap 40 --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -12,5 +12,4 @@
 
 This paragraph has multiline code `{{
   render_customer_notice( customer.account_name,  customer.preferred_delivery_address )
-}}` followed by ordinary prose that
-still wraps at the requested column.
+}}` followed by ordinary prose that still wraps at the requested column.
```

```json
{
  "stdin": "This paragraph before the multiline template has enough ordinary prose to wrap at the requested column.\n\n{{\nrender_customer_notice(\n  customer.account_name,\n  customer.preferred_delivery_address,\n  \"An overwide fallback message that must stay exactly as authored\"\n)\n}}\n\nThis paragraph has multiline code `{{\n  render_customer_notice( customer.account_name,  customer.preferred_delivery_address )\n}}` followed by ordinary prose that still wraps at the requested column.\n",
  "reviewed_stdout": "This paragraph before the multiline\ntemplate has enough ordinary prose to\nwrap at the requested column.\n\n{{\nrender_customer_notice(\n  customer.account_name,\n  customer.preferred_delivery_address,\n  \"An overwide fallback message that must stay exactly as authored\"\n)\n}}\n\nThis paragraph has multiline code `{{\n  render_customer_notice( customer.account_name,  customer.preferred_delivery_address )\n}}` followed by ordinary prose that\nstill wraps at the requested column.\n",
  "current_stdout": "This paragraph before the multiline\ntemplate has enough ordinary prose to\nwrap at the requested column.\n\n{{\nrender_customer_notice(\n  customer.account_name,\n  customer.preferred_delivery_address,\n  \"An overwide fallback message that must stay exactly as authored\"\n)\n}}\n\nThis paragraph has multiline code `{{\n  render_customer_notice( customer.account_name,  customer.preferred_delivery_address )\n}}` followed by ordinary prose that still wraps at the requested column.\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-454

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-454) · [active transcript](../../cases/recovered_pr4_markdown_template_prose_boundaries.case) · [reviewed transcript](recovered_pr4_markdown_template_prose_boundaries.case)

```text
format --stdin-file-path input.md --wrap sentence --verify
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,4 +1,5 @@
-> Before this {{ foo }} after.
+> Before
+> this {{ foo }} after.
 
 Before
 <span>{{ foo }}</span> after.
@@ -9,19 +10,26 @@
 
 Before {{ foo }} after.
 
-Before *`<% keep   this %>`* after.
+Before
+*`<% keep   this %>`* after.
 
-Before [`<% keep   this %>`](url) after.
+Before
+[`<% keep   this %>`](url) after.
 
-Before [[[[{{ foo }}]]]] after.
+Before
+[[[[{{ foo }}]]]] after.
 
-Before [[[[`<% keep   this %>`]]]] after.
+Before
+[[[[`<% keep   this %>`]]]] after.
 
-Before [outer [middle [inner {{ keep   this }}] after] end] after.
+Before
+[outer [middle [inner {{ keep   this }}] after] end] after.
 
-Before [outer *[[`<% keep   this %>`]]* after] tail.
+Before
+[outer *[[`<% keep   this %>`]]* after] tail.
 
-Before [outer [[label](url "{{ keep   this }}")] after] tail.
+Before
+[outer [[label](url "{{ keep   this }}")] after] tail.
 
 Following prose.
 
```

```json
{
  "stdin": "> Before\n> this {{ foo }} after.\n\nBefore\n<span>{{ foo }}</span> after.\n\nBefore\n{{ foo }}\nafter.\n\nBefore\n{{ foo }} after.\n\nBefore\n*`<% keep   this %>`* after.\n\nBefore\n[`<% keep   this %>`](url) after.\n\nBefore\n[[[[{{ foo }}]]]] after.\n\nBefore\n[[[[`<% keep   this %>`]]]] after.\n\nBefore\n[outer [middle [inner {{ keep   this }}] after] end] after.\n\nBefore\n[outer *[[`<% keep   this %>`]]* after] tail.\n\nBefore\n[outer [[label](url \"{{ keep   this }}\")] after] tail.\n\nFollowing\nprose.\n\n[^note]: First paragraph.\n\n    Another   paragraph.\n\n    ```python\n    print(\"keep   this\")\n    ```\n",
  "reviewed_stdout": "> Before this {{ foo }} after.\n\nBefore\n<span>{{ foo }}</span> after.\n\nBefore\n{{ foo }}\nafter.\n\nBefore {{ foo }} after.\n\nBefore *`<% keep   this %>`* after.\n\nBefore [`<% keep   this %>`](url) after.\n\nBefore [[[[{{ foo }}]]]] after.\n\nBefore [[[[`<% keep   this %>`]]]] after.\n\nBefore [outer [middle [inner {{ keep   this }}] after] end] after.\n\nBefore [outer *[[`<% keep   this %>`]]* after] tail.\n\nBefore [outer [[label](url \"{{ keep   this }}\")] after] tail.\n\nFollowing prose.\n\n[^note]:\n    First paragraph.\n\n    Another paragraph.\n\n    ```python\n    print(\"keep   this\")\n    ```\n",
  "current_stdout": "> Before\n> this {{ foo }} after.\n\nBefore\n<span>{{ foo }}</span> after.\n\nBefore\n{{ foo }}\nafter.\n\nBefore {{ foo }} after.\n\nBefore\n*`<% keep   this %>`* after.\n\nBefore\n[`<% keep   this %>`](url) after.\n\nBefore\n[[[[{{ foo }}]]]] after.\n\nBefore\n[[[[`<% keep   this %>`]]]] after.\n\nBefore\n[outer [middle [inner {{ keep   this }}] after] end] after.\n\nBefore\n[outer *[[`<% keep   this %>`]]* after] tail.\n\nBefore\n[outer [[label](url \"{{ keep   this }}\")] after] tail.\n\nFollowing prose.\n\n[^note]:\n    First paragraph.\n\n    Another paragraph.\n\n    ```python\n    print(\"keep   this\")\n    ```\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-456

The narrower #9/#10 policy retains this template context or layout. See the boundary notes in the index; this does not declare #4 undesirable.

[Mapping](mapping.md#pr4-456) · [active transcript](../../cases/recovered_pr4_markdown_braced_template_spans_allow_heading_formatting_456.case) · [reviewed transcript](recovered_pr4_markdown_braced_template_spans_allow_heading_formatting_456.case)

```text
format --stdin-file-path input.md --wrap none
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,5 +1,6 @@
-# Title {{ keep   spacing }}
+#   Title {{ keep   spacing }}   ##
 
-# Setext {{ keep   spacing }}
+Setext {{ keep   spacing }}
+====
 
 # Normal
```

```json
{
  "stdin": "#   Title {{ keep   spacing }}   ##\n\nSetext {{ keep   spacing }}\n====\n\n#   Normal ##\n",
  "reviewed_stdout": "# Title {{ keep   spacing }}\n\n# Setext {{ keep   spacing }}\n\n# Normal\n",
  "current_stdout": "#   Title {{ keep   spacing }}   ##\n\nSetext {{ keep   spacing }}\n====\n\n# Normal\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

## pr4-457

Main still preserves shortcode bodies; #4 formats intervening Markdown. Tag/body policy remains a maintainer decision.

[Mapping](mapping.md#pr4-457) · [active transcript](../../cases/recovered_pr4_markdown_hugo_shortcode_tags_preserve_only_the_tag_457.case) · [reviewed transcript](recovered_pr4_markdown_hugo_shortcode_tags_preserve_only_the_tag_457.case)

```text
format --stdin-file-path input.md --wrap sentence
```

```diff
--- reviewed stdout
+++ main stdout
@@ -1,3 +1,3 @@
 {{< notice >}}
-This Markdown body should be formatted.
+This    Markdown body should be formatted.
 {{< /notice >}}
```

```json
{
  "stdin": "{{< notice >}}\nThis    Markdown body should be formatted.\n{{< /notice >}}\n",
  "reviewed_stdout": "{{< notice >}}\nThis Markdown body should be formatted.\n{{< /notice >}}\n",
  "current_stdout": "{{< notice >}}\nThis    Markdown body should be formatted.\n{{< /notice >}}\n",
  "reviewed_stderr": "",
  "current_stderr": "",
  "reviewed_status": 0,
  "current_status": 0
}
```

