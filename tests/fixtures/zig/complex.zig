const std = @import("std");
const math = @import("math.zig");

pub const Calculator = struct {
    value: i32,

    pub fn init() Calculator {
        return Calculator{ .value = 0 };
    }

    pub fn add(self: *Calculator, a: i32, b: i32) i32 {
        self.value = a + b;
        return self.value;
    }

    pub fn print(self: *Calculator, message: []const u8) void {
        std.debug.print("{s}\n", .{message});
    }
};

pub const Point = struct {
    x: f64,
    y: f64,

    pub fn distanceTo(self: Point, other: Point) f64 {
        const dx = self.x - other.x;
        const dy = self.y - other.y;
        return @sqrt(dx * dx + dy * dy);
    }
};

pub const Color = enum {
    Red,
    Green,
    Blue,
};

pub const MAX_SIZE: usize = 100;

test "addition" {
    var calc = Calculator.init();
    const result = calc.add(2, 3);
    try std.testing.expectEqual(5, result);
}

pub fn main() void {
    var calc = Calculator.init();
    const result = calc.add(5, 10);
    std.debug.print("{}\n", .{result});
}
