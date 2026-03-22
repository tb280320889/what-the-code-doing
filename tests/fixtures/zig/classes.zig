const std = @import("std");

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
};

pub const Color = enum {
    Red,
    Green,
    Blue,
};

pub const MAX_SIZE: usize = 100;
