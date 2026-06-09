local function newPart(name: string, position: Vector3, size: Vector3): Part
    local part: Part = Part.new()
    part.Name = name
    part.Parent = workspace
    part.Size = size
    part.Position = position
    return part
end

local subject = newPart("Subject", Vector3.zero, Vector3.one)
subject.Transparency = 1

Camera:SetSubject(subject)

local floor = newPart("Floor", Vector3.zero, Vector3.new(5, 0.1, 5))
floor.Color = Color3.fromRGB(20, 200, 20)
local floor_collider = Collider.new()
floor_collider.Parent = floor

local part = Part.new()
part.Name = "Part"
part.Position = Vector3.new(0, 10, 0)
part.Color = Color3.fromRGB(200, 20, 20)

local part_collider = Collider.new()
part_collider.Parent = part
local part_rb = Rigidbody.new()
part_rb.Parent = part

task.wait(2)

part.Parent = workspace

task.wait(2)

local clone_part = part:Clone()
clone_part.Parent = workspace
clone_part.Position = Vector3.new(0, 10, 0)
