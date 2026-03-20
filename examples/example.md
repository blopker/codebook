# Markdown Injection Test File

This file exercises multi-language injection and edge cases.

## Basic prose with typos

I'm bvd at splellin. Some regulr text with misspeled words.

## Fenced code blocks with known languages

Python function with a typo in the name:

```python
# A coment about the functin
def calculat_user_age(bith_date: str) -> int:
    """Calculaet the user's age from a date strng."""
    user_age = get_current_date() - bith_date
    return user_age
```

Bash commands (bash.scm doesn't capture command invocations):

```bash
# Instal dependencies for the proejct
mkdir -p build/outpt
cp src/*.rs build/outpt/
echo "Doen compiling"
```

Rust with various identifier types:

```rust
// Calclate the user's scroe
fn calculat_score(usr_input: &str) -> u32 {
    let mut scroe = 0;
    let mesage = "Helo Wolrd";
    scroe
}
```

JavaScript with strings and comments:

```javascript
// Procss the usr requet
function handl_request(requst) {
    const mesage = "Somthing went wrng";
    return mesage;
}
```

## Language aliases

Common aliases should resolve correctly:

```py
def wrld_functin():
    pass
```

```js
function wrld_functin() {}
```

```rs
fn wrld_functin() {}
```

```sh
# A coment in shell
echo "doen"
```

```ts
function wrld_functin(): void {}
```

## Unknown and missing language tags

Unknown language (should be completely skipped):

```fortran
      PROGRAM BADSPELIN
      WRITE(*,*) 'This shuld not be chekced'
      END
```

No language tag (should be completely skipped):

```
badwwword and uncheckedtypo should not appear
```

## Block-level HTML (injected as HTML)

<div class="containr">
  <p>This paragraf has a misspeled word inside HTML.</p>
  <span>Another sentance with erors.</span>
</div>

## Inline HTML in prose (not separately injected)

Some text with <strong>boldd</strong> and <em>italc</em> formatting.

## Block quotes

> A block quoet with misspeled words should be chekced normally.

Nested block quote:

> > A neested quoet inside another quoet.

## Many code blocks (stress test for recursion)

```python
x = "frst"
```

```bash
echo "secnd"
```

```python
y = "thrd"
```

```rust
let z = "fouth";
```

```javascript
let w = "fifh";
```

```python
a = "sixh"
```

## Mixed content ordering

Text before. Then code:

```python
def befre_and_aftr():
    pass
```

Text between code blocks with a tyypo.

```rust
// Anothr comment
fn middl_function() {}
```

More text aftr the code.

## Edge cases

Empty code block with language:

```python
```

Single-line code block:

```python
x = "singel"
```

Code block at end of file with no trailing newline:

```python
y = "finl"
```
