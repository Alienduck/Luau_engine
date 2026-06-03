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

local accumulator = 0

local connection = RunService.RenderStepped:Connect(function(dt)
    accumulator += dt
    if accumulator >= 1.0 then
        accumulator -= 1.0

        local falling_part = Part.new()
        falling_part.Size = Vector3.new(1, 1, 1)
        falling_part.Color = Color3.fromRGB(100, 100, 255)
        falling_part.Position = Vector3.new(math.random(-4, 4), 5, math.random(-4, 4))
        falling_part.Transparency = math.random(0, 100) / 100

        local col = Collider.new()
        col.Parent = falling_part
        local rb = Rigidbody.new()
        rb.Parent = falling_part
    end
end)

task.wait(5)
print("disconnect")
connection:Disconnect()
