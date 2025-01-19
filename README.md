# Noisy Image Generator

## Why does this exist?

I recently got a new computer, and while setting it up, I decided I wanted to change my wallpaper
to make the computer _feel_ new. Unfortunately, the pictures I already had laying around didn't
actually look that good. So I began looking. I found inspiration in a YouTube thumbnail I saw.
It had a gradient background and then a circle with some grain applied to it, and I don't know
it just spoke to me.

And that brings to this project. I loved the general concept, but the colors didn't fit with
what I was really going for, so I figured that it couldn't be that hard to make for myself, so I made this!

## .noisy input files

Imagine this program like a painter. You start by setting up your canvas, and then you 
progressively draw images on top of each other until you reach a finished product.

More concretely, let's take a look at how things work.


The start of every .noisy file is a canvas element. You 
width/height and color like so: 

```
canvas {
    width 1920
    height 1080
    color #000000
}
```
This is a required block, but notice that it's the only required block. Height and width are
required, but color is not, and and the default value is black, so the above 

```
#define complex-shape parameter1 parameter2 {
    circle {
        height parameter1
        width parameter2
        radius parameter1*parameter2
    }
}

```


```
#constant name=value
```

### Data types

| Variable types | Allowed values |
|---|---|
|color| 32-bit RGB hex code: `#RRGGBB`|
|canvas x|a whole number `x` where `0 <= x < canvas.width` |
|canvas y|a whole number `y` where `0 <= y < canvas.height`|


in general: 
    expression: integer | hex value | name
    >
    const: const name equals expression
    write: block-title LBrace (property expression)* RBrace
    >
    define: define block-title param* LBrace write RBrace


