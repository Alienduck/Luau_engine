-- Subtle reference ^^
local reactive = {}

export type State<T> = {
	value: T,
	listeners: { (new_value: T) -> () },
	get: (self: State<T>) -> T,
	set: (self: State<T>, new_value: T) -> (),
	on_change: (self: State<T>, callback: (new_value: T) -> ()) -> (),
}

function reactive.State<T>(initialValue): State<T>
	local self = {
		value = initialValue,
		listeners = {},
	}

	function self:get()
		return self.value
	end

	function self:set(new_value)
		if self.value ~= new_value then
			self.value = new_value
			for _, listener in ipairs(self.listeners) do
				listener(new_value)
			end
		end
	end

	function self:on_change(callback)
		table.insert(self.listeners, callback)
		callback(self.value)
	end

	return self :: State<T>
end

return reactive
