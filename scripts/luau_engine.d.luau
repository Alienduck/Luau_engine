-- luau_engine.d.luau
-- Type declarations for all globals injected by the Rust engine.

-- ─────────────────────────────────────────────
-- Primitive types
-- ─────────────────────────────────────────────

export type Vector3 = {
    X: number, Y: number, Z: number,
    Dot: (self: Vector3, other: Vector3) -> number,
    Cross: (self: Vector3, other: Vector3) -> Vector3,
    Magnitude: (self: Vector3) -> number,
    Normalize: (self: Vector3) -> Vector3,
    Lerp: (self: Vector3, goal: Vector3, alpha: number) -> Vector3,
}

export type Color3 = {
    R: number, G: number, B: number,
    Lerp: (self: Color3, goal: Color3, alpha: number) -> Color3,
}

export type CFrame = {
    X: number, Y: number, Z: number,
    Position: Vector3,
    LookVector: Vector3,
    RightVector:Vector3,
    UpVector: Vector3,
    Inverse: (self: CFrame) -> CFrame,
    Lerp: (self: CFrame, goal: CFrame, alpha: number) -> CFrame,
    ToEulerAnglesYXZ: (self: CFrame) -> (number, number, number),
}

export type Udim2 = {
    -- x_scale: number,
    -- y_scale: number,
    -- x_offset: number,
    -- y_offset: number
}

-- ─────────────────────────────────────────────
-- Instance types
-- ─────────────────────────────────────────────

export type Part = {
    Position: Vector3,
    Size: Vector3,
    Handle: number,
    Color: Color3,
    Transparency: number,
    Destroy: (self: Part) -> (),
}

export type Frame<t> = {
    Size: Udim2,
    Position: Udim2,
    BackgroundColor3: Color3,
    Transparency: number,
    Parent: t,
}

export type ScreenGui = {}

export type Camera = {
    FirstPerson: boolean,
    Distance: number,
    Sensitivity: number,
    Fov: number,
    CFrame: CFrame,
    SetSubject: (self: Camera, part: Part) -> (),
    ClearSubject: (self: Camera) -> (),
    SetLookBinding: (self: Camera, device: string, name: string) -> (),
    SetLockBinding: (self: Camera, device: string, name: string) -> (),
}

export type Rigidbody<t> = {
    Parent: t
}

export type Collider<t> = {
    Parent: t
}

export type RBXScriptConnection = {
    Disconnect: (self: RBXScriptConnection) -> (),
}

export type RBXScriptSignal<T...> = {
    Connect: (self: RBXScriptSignal<T...>, callback: (T...) -> ()) -> RBXScriptConnection,
}

-- ─────────────────────────────────────────────
-- Services types
-- ─────────────────────────────────────────────

export type UserInputService = {
    InputBegan: RBXScriptSignal<string, string>,
    InputEnded: RBXScriptSignal<string, string>,
}

export type RunService = {
    RenderStepped: RBXScriptSignal<number>,
}

-- ─────────────────────────────────────────────
-- Global constructors
-- ─────────────────────────────────────────────

declare Vector3: {
    new: (x: number, y: number, z: number) -> Vector3,
    zero: Vector3,
    one: Vector3,
}

declare Color3: {
    new: (r: number, g: number, b: number) -> Color3,
    fromRGB: (r: number, g: number, b: number) -> Color3,
}

declare CFrame: {
    new: (x: number, y: number, z: number) -> CFrame,
    lookAt: (position: Vector3, target: Vector3) -> CFrame,
    fromEulerAnglesYXZ:(rx: number, ry: number, rz: number) -> CFrame,
    identity: CFrame,
}

declare Udim2: {
    new: (xs: number, ys: number, xo: number, yo: number) -> Udim2,
    fromScale: (xs: number, ys: number) -> Udim2,
    fromOffset: (xo: number, yo: number) -> Udim2,
}

declare Part: {
    new: () -> Part,
}

declare Frame: {
    new: <t>() -> Frame<t>,
}

declare ScreenGui: {
    new: () -> ScreenGui,
}

declare Rigidbody: {
    new: <t>() -> Rigidbody<t>
}

declare Collider: {
    new: <t>() -> Collider<t>
}

--- Singleton camera — one per scene, configured from Luau.
declare Camera: Camera

-- ─────────────────────────────────────────────
-- task library
-- ─────────────────────────────────────────────

declare task: {
    wait: (t: number?) -> (),
    spawn: <T...>(f: (T...) -> (), T...) -> thread,
    defer: <T...>(f: (T...) -> (), T...) -> thread,
}

-- Services

declare UserInputService: UserInputService

declare RunService: RunService
