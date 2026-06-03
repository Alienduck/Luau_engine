local subject = Part.new()
subject.Position = Vector3.new(0, 0, 0)
subject.Size = Vector3.new(1, 1, 1)

Camera.Distance = 10
Camera.Sensitivity = 1.2
Camera.Fov = 70
Camera:SetSubject(subject)

local floor = Part.new()
floor.Size = Vector3.new(10, 1, 10)
floor.Position = Vector3.new(0, 0, 0)

local floor_collider = Collider.new()
floor_collider.Parent = floor

while true do
    task.wait(1)
    local falling_part = Part.new()
    falling_part.Size = Vector3.new(1, 1, 1)
    falling_part.Color = Color3.fromRGB(100, 100, 255)
    falling_part.Position = Vector3.new(math.random(-4, 4), 5, math.random(-4, 4))
    falling_part.Transparency = math.random(0, 100) / 100

    local fp_collider = Collider.new()
    fp_collider.Parent = falling_part

    local fp_rb = Rigidbody.new()
    fp_rb.Parent = falling_part
end
