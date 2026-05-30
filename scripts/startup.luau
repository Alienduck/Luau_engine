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

ContextActionService:BindAction("CameraToggleLock", "Mouse", "left")

-- NOT WORKING
-- task.spawn(function()
--     local t = 0
--     while true do
--         task.wait(1/60)
--         t += 1/60
--         subject.Position = Vector3.new(
--             math.sin(t * 0.5) * 3,
--             1 + math.sin(t) * 0.5,
--             math.cos(t * 0.5) * 3
--         )
--         subject.Color = Color3.new(
--             0.3 + math.sin(t) * 0.3,
--             0.6,
--             0.9
--         )
--     end
-- end)

-- task.spawn(function()
--     while true do
--         task.wait(1)
--         local p = subject.Position
--         print(string.format("Position: (%.2f, %.2f, %.2f)", p.X, p.Y, p.Z))
--     end
-- end)
