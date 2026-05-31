-- Spawn a subject part that bobs up and down
local subject = Part.new()
subject.Position = Vector3.new(0, 1, 0)
subject.Color = Color3.fromRGB(80, 160, 255)
subject.Size = Vector3.new(1, 2, 1)

-- Spawn a floor
local floor = Part.new()
floor.Position = Vector3.new(0, -0.5, 0)
floor.Size = Vector3.new(20, 1, 20)
floor.Color = Color3.fromRGB(60, 60, 60)

-- Attach camera to the subject
-- Right-click + drag to orbit  |  Shift to toggle mouse lock
Camera.Distance = 8
Camera.Sensitivity = 1.2
Camera.Fov = 70
Camera:SetSubject(subject)

task.wait(2)

local function test_context_action(str: string)
    print("F press, receive -> " .. str)
    Camera.FirstPerson = not Camera.FirstPerson
end

ContextActionService:BindAction("TestAction", test_context_action, "Keyboard", "f")

ContextActionService:BindAction("Forward", function()
    local look_vector = Camera.CFrame.LookVector
    subject.Position += look_vector
end, "Keyboard", "w")

ContextActionService:BindAction("Backward", function()
    local look_vector = Camera.CFrame.LookVector
    subject.Position -= look_vector
end, "Keyboard", "s")

ContextActionService:BindAction("Left", function()
    local right_vector = Camera.CFrame.RightVector
    subject.Position -= right_vector
end, "Keyboard", "a")

ContextActionService:BindAction("Right", function()
    local right_vector = Camera.CFrame.RightVector
    subject.Position += right_vector
end, "Keyboard", "d")

ContextActionService:BindAction("test", function()
    print(Camera.CFrame)
end, "Keyboard", "e")
