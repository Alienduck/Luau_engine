export type Vector3 = {
    X: number,
    Y: number,
    Z: number,
}

export type Color3 = {
    R: number,
    G: number,
    B: number,
    Lerp: (self: Color3, goal: Color3, alpha: number) -> Color3,
}

export type Part = {
    Position: Vector3,
    Size:     Vector3,
    Color:    Color3,
    Destroy:  (self: Part) -> (),
}

declare Vector3: {
    new:  (x: number, y: number, z: number) -> Vector3,
    zero: Vector3,
    one:  Vector3,
}

declare Color3: {
    new:     (r: number, g: number, b: number) -> Color3,
    fromRGB: (r: number, g: number, b: number) -> Color3,
}

declare Part: {
    new: () -> Part,
}

declare task: {
    --- Yield the current coroutine for `t` seconds (default 0).
    wait:  (t: number?) -> (),
    --- Spawn `f` as a new coroutine and resume it immediately.
    spawn: <T...>(f: (T...) -> (), T...) -> thread,
    --- Like spawn but resumes on the next frame.
    defer: <T...>(f: (T...) -> (), T...) -> thread,
}
