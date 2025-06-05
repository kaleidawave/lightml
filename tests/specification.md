> TODO probably a separate file for retrieval

### Elements

```html
<h1>Hello world</h1>
```

Should parse to

```
Ok(
    Element {
        tag_name: "h1",
        attributes: [],
        children: Children(
            [
                TextNode(
                    "Hello world",
                ),
            ],
        ),
    },
)
```

### Self closing

```html
<div>
    <img src="">
</div>
```

Should parse to

```
Ok(
    Element {
        tag_name: "div",
        attributes: [],
        children: Children(
            [
                Element(
                    Element {
                        tag_name: "img",
                        attributes: [
                            Attribute {
                                key: "src",
                                value: "",
                            },
                        ],
                        children: SelfClosing,
                    },
                ),
            ],
        ),
    },
)
```

### Attributes

```html
<h1 data-x class="x">Hello world</h1>
```

Should parse to

```
Ok(
    Element {
        tag_name: "h1",
        attributes: [
            Attribute {
                key: "data-x",
                value: "",
            },
            Attribute {
                key: "class",
                value: "x",
            },
        ],
        children: Children(
            [
                TextNode(
                    "Hello world",
                ),
            ],
        ),
    },
)
```

#### Empty attributes

```html
<h1 item>Hello world</h1>
```

```
Ok(
    Element {
        tag_name: "h1",
        attributes: [
            Attribute {
                key: "item",
                value: "",
            },
        ],
        children: Children(
            [
                TextNode(
                    "Hello world",
                ),
            ],
        ),
    },
)
```

#### Attribute value delimeters

```html
<h1 item=something>Hello world</h1>
```

```
Ok(
    Element {
        tag_name: "h1",
        attributes: [
            Attribute {
                key: "item",
                value: "something",
            },
        ],
        children: Children(
            [
                TextNode(
                    "Hello world",
                ),
            ],
        ),
    },
)
```

### Element children

```html
<div>
	<h1>Hello ya!</h1>
</div>
```

Should parse to

```
Ok(
    Element {
        tag_name: "div",
        attributes: [],
        children: Children(
            [
                Element(
                    Element {
                        tag_name: "h1",
                        attributes: [],
                        children: Children(
                            [
                                TextNode(
                                    "Hello ya!",
                                ),
                            ],
                        ),
                    },
                ),
            ],
        ),
    },
)
```

### `script` tag

```html
<div>
	<script>
		const x = `<h1>Hiya</h1>`;
	</script>
</div>
```

Should parse to

```
Ok(
    Element {
        tag_name: "div",
        attributes: [],
        children: Children(
            [
                Element(
                    Element {
                        tag_name: "script",
                        attributes: [],
                        children: Literal(
                            "\n\t\tconst x = `<h1>Hiya</h1>`;\n\t",
                        ),
                    },
                ),
            ],
        ),
    },
)
```

### Custom elements

```html
<element-x></element-x>
```

```
Ok(
    Element {
        tag_name: "element-x",
        attributes: [],
        children: Children(
            [],
        ),
    },
)
```

### Implicit tags

From the HTML specification

#### Paragraph elements

> A `p` element's end tag may be omitted if the `p` element is immediately followed by an *1 element, or if there is no more content in the parent element and the parent element is an HTML element that is **not** an *2.

> *1 = `address`, `article`, `aside`, `blockquote`, `details`, `dialog`, `div`, `dl`, `fieldset`, `figcaption`, `figure`, `footer`, `form`, `h1`, `h2`, `h3`, `h4`, `h5`, `h6`, `header`, `hgroup`, `hr`, `main`, `menu`, `nav`, `ol`, `p`, `pre`, `search`, `section`, `table`, `ul`

> *2 = `a`, `audio`, `del`, `ins`, `map`, `noscript`, `video`, `*-*` autonomous custom element (I think)


```html
<div>
    <p>Hiya
    <p>Hello
</div>
```

```
Ok(
    Element {
        tag_name: "div",
        attributes: [],
        children: Children(
            [
                Element(
                    Element {
                        tag_name: "p",
                        attributes: [],
                        children: Children(
                            [
                                TextNode(
                                    "Hiya\n    ",
                                ),
                            ],
                        ),
                    },
                ),
                Element(
                    Element {
                        tag_name: "p",
                        attributes: [],
                        children: Children(
                            [
                                TextNode(
                                    "Hello\n",
                                ),
                            ],
                        ),
                    },
                ),
            ],
        ),
    },
)
```

#### List elements

```html
<ul>
    <li>Hiya
    <li>Hello
</ul>
```

```
Ok(
    Element {
        tag_name: "ul",
        attributes: [],
        children: Children(
            [
                Element(
                    Element {
                        tag_name: "li",
                        attributes: [],
                        children: Children(
                            [
                                TextNode(
                                    "Hiya\n    ",
                                ),
                            ],
                        ),
                    },
                ),
                Element(
                    Element {
                        tag_name: "li",
                        attributes: [],
                        children: Children(
                            [
                                TextNode(
                                    "Hello\n",
                                ),
                            ],
                        ),
                    },
                ),
            ],
        ),
    },
)
```

### `DOCTYPE` tag

> [See](https://html.spec.whatwg.org/multipage/syntax.html#the-doctype)

```html
<!doCType html>
<html></html>
```

Recieved

```
Ok(
    Element {
        tag_name: "html",
        attributes: [],
        children: Children(
            [],
        ),
    },
)
```

### Comments

#### Comment in structure

```html
<p>
    <!-- i am a comment -->
</p>
```

```
Ok(
    Element {
        tag_name: "p",
        attributes: [],
        children: Children(
            [
                Comment(
                    " i am a comment ",
                ),
            ],
        ),
    },
)
```

#### Weird comments

```html
<div><!--My favorite operators are > and <!--></div>
```

```
Ok(
    Element {
        tag_name: "div",
        attributes: [],
        children: Children(
            [
                Comment(
                    "My favorite operators are > and <!",
                ),
            ],
        ),
    },
)
```

### Whitespace

```html
<p>This is some <strong>text</strong> <em>here</em></p>
```

```
Ok(
    Element {
        tag_name: "p",
        attributes: [],
        children: Children(
            [
                TextNode(
                    "This is some ",
                ),
                Element(
                    Element {
                        tag_name: "strong",
                        attributes: [],
                        children: Children(
                            [
                                TextNode(
                                    "text",
                                ),
                            ],
                        ),
                    },
                ),
                TextNode(
                    " ",
                ),
                Element(
                    Element {
                        tag_name: "em",
                        attributes: [],
                        children: Children(
                            [
                                TextNode(
                                    "here",
                                ),
                            ],
                        ),
                    },
                ),
            ],
        ),
    },
)
```

...