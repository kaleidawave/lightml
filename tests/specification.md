> TODO probably a separate file for retrieval

### Basic

```html
<h1>Hello world</h1>
```

Should produce

```
Ok(
    Document {
        html_element: Element {
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
    },
)
```

### Attributes

#TODO Implicit, in quotes out of quotes etc. Across lines etc. Want 10 sections.

```html
<h1 data-x class="x">Hello world</h1>
```

Should produce

```
Ok(
    Document {
        html_element: Element {
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
    },
)
```

### Nested

```html
<div>
	<h1>Hello ya!</h1>
</div>
```

Should produce

```
Ok(
    Document {
        html_element: Element {
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

Should produce

```
Ok(
    Document {
        html_element: Element {
            tag_name: "div",
            attributes: [],
            children: Children(
                [
                    Element(
                        Element {
                            tag_name: "script",
                            attributes: [],
                            children: Literal(
                                "\r\n\t\tconst x = `<h1>Hiya</h1>`;\r\n\t",
                            ),
                        },
                    ),
                ],
            ),
        },
    },
)
```

### Custom elements

```html
<element-x></element-x>
```

```
Ok(
    Document {
        html_element: Element {
            tag_name: "element-x",
            attributes: [],
            children: Children(
                [],
            ),
        },
    },
)
```

### Implicit tags

From the HTML specification

> A p element's end tag may be omitted if the p element is immediately followed by an *1 element, or if there is no more content in the parent element and the parent element is an HTML element that is **not** an *2.

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
    Document {
        html_element: Element {
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
                                        "Hiya\r\n    ",
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
                                        "Hello\r\n",
                                    ),
                                ],
                            ),
                        },
                    ),
                ],
            ),
        },
    },
)
```

### `DOCTYPE` tag

See https://html.spec.whatwg.org/multipage/syntax.html#the-doctype

```html
<!doCType html>
<html></html>
```

Recieved

```
Ok(
    Document {
        html_element: Element {
            tag_name: "html",
            attributes: [],
            children: Children(
                [],
            ),
        },
    },
)
```

...