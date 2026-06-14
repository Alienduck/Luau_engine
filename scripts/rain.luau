local module = {}

type PartParams = {
    name: string?,
    position: Vector3?,
    size: Vector3?,
    color: Color3?,
    transparency: number?,
    parent: Instance?,
    has_collider: boolean?,
    has_rigidbody: boolean?
}

function __spawn_part(params: PartParams): Part
    local part = Part.new()
    part.Parent = params.parent or nil
    part.Name = params.name or "Part"
    part.Position = params.position or Vector3.zero
    part.Size = params.size or Vector3.one
    part.Color = params.color or Color3.new(1, 1, 1)
    part.Transparency = params.transparency or 0

    if params.has_collider then
        local collider = Collider.new()
        collider.Parent = part
    end

    if params.has_rigidbody then
        local rb = Rigidbody.new()
        rb.Parent = part
    end

    return part
end

function module.Start()
    local _floor = __spawn_part({
        name = "floor",
        position = Vector3.zero,
        size = Vector3.new(20, 0.1, 20),
        color = Color3.new(1, 1, 1),
        parent = workspace,
        has_collider = true
    })

    local count = 0
    local time_before_next = math.random(0, 1000) / 1000

    RunService.RenderStepped:Connect(function(delta: number)
        count += delta
        if count < time_before_next then return end

        count = 0
        time_before_next = math.random(0, 500) / 10000

        local part = __spawn_part({
            position = Vector3.new(math.random(-10, 10), math.random(7, 13), math.random(-10, 10)),
            size = Vector3.new(1 / 10, math.random(2, 5) / 10, 1 / 10),
            color = Color3.fromRGB(50, 50, math.random(200, 255)),
            transparency = math.random(50, 100) / 100,
            parent = workspace,
            has_collider = true,
            has_rigidbody = true
        })
        part.Material = "Neon"

        part.Touched:Connect(function(hit: Instance)
            if hit.Name == "floor" then
                part:Destroy()
            end
        end)
    end)
end

return module
