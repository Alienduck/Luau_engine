local subject = Part.new()
subject.Position = Vector3.new(0, 2, 0)
subject.Size = Vector3.new(1, 1, 1)
subject.Color = Color3.fromRGB(100, 0, 0)
subject.Transparency = 1

Camera.Distance = 12
Camera.Sensitivity = 1.2
Camera.Fov = 70
Camera:SetSubject(subject)

local floor = Part.new()
floor.Size = Vector3.new(10, 1, 10)
floor.Position = Vector3.new(0, 0, 0)

local floor_collider = Collider.new()
floor_collider.Parent = floor

local accumulator = 0

local connection = RunService.RenderStepped:Connect(function(dt: number)
    accumulator += dt
    if accumulator >= 1.0 then
        accumulator -= 1.0

        local falling_part = Part.new()
        local random_size = math.random(1, 2)
        falling_part.Size = Vector3.new(random_size, random_size, random_size)
        falling_part.Color = Color3.fromRGB(100, 100, 255)
        falling_part.Position = Vector3.new(math.random(-4, 4), 5, math.random(-4, 4))
        falling_part.Transparency = math.random(0, 100) / 100

        local col = Collider.new()
        col.Parent = falling_part
        local rb = Rigidbody.new()
        rb.Parent = falling_part
    end
end)

UserInputService.InputBegan:Connect(function(key: string, device: string)
    if key == "KeyF" then Camera.FirstPerson = not Camera.FirstPerson end
end)

local screen_gui = ScreenGui.new()

local container = Frame.new()
container.Parent = screen_gui
container.Position = Udim2.fromScale(.25, .25)
container.Size = Udim2.fromScale(.5, .5)
container.BackgroundColor3 = Color3.fromRGB(200, 50, 50)
container.Transparency = 0.8

local test = require("scripts/test_require")
task.wait(10)
connection:Disconnect()
