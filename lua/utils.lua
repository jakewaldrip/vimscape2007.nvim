---@class Utils
---@field round function round to the nearest whole number
---@field dump function print table recursively
---@field print_to_buffer function print the content to the buffer, accepts table of lines
local M = {}

---@return number
M.round = function(float)
	return math.floor(float + 0.5)
end

M.dump = function(o)
	if type(o) == "table" then
		local s = "{ "
		for k, v in pairs(o) do
			if type(k) ~= "number" then
				k = '"' .. k .. '"'
			end
			s = s .. "[" .. k .. "] = " .. M.dump(v) .. ","
		end
		return s .. "} "
	else
		return tostring(o)
	end
end

M.print_to_buffer = function(content, bufnr)
	for k, v in pairs(content) do
		local text = {}
		text[1] = v
		vim.api.nvim_buf_set_lines(bufnr, k, k, false, text)
	end
end

return M
