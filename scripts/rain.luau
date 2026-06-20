local module = {}

type PartParams = {
	name: string?,
	position: Vector3?,
	size: Vector3?,
	color: Color3?,
	transparency: number?,
	parent: Instance?,
	has_collider: boolean?,
	has_rigidbody: boolean?,
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

local PARAMS = {
	POS_XZ = function()
		return math.random(-100, 100)
	end,
	POS_Y = function()
		return math.random(20, 50)
	end,
}

function module.Start()
	local count = 0

	RunService.RenderStepped:Connect(function(delta: number)
		count += delta

		count = 0
		local part = __spawn_part({
			position = Vector3.new(PARAMS.POS_XZ(), PARAMS.POS_Y(), PARAMS.POS_XZ()),
			size = Vector3.new(1 / 15, math.random(2, 5) / 10, 1 / 15),
			color = Color3.fromRGB(50, 50, math.random(200, 255)),
			transparency = math.random(50, 100) / 100,
			parent = workspace,
			has_collider = true,
			has_rigidbody = true,
		})
		part.Material = "Neon"

		part.Touched:Connect(function(hit: Instance)
			part:Destroy()
		end)
	end)
end

return module
