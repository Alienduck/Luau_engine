local module = {}

export type EntityModule = {
	part: Part,
	rb: Rigidbody,
	collider: Collider,
} & typeof(module)

export type Params = {
	position: Vector3?,
}

function module.new(params: Params): EntityModule
	local self = {} :: EntityModule
	local part = Part.new()
	part.Parent = workspace
	part.Position = params.position or Vector3.zero
	self.part = part

	local rb = Rigidbody.new()
	rb.Parent = part
	rb.Mass = 10
	self.rb = rb

	local collider = Collider.new()
	collider.Parent = part
	self.collider = collider

	return setmetatable(self, module)
end

return module
