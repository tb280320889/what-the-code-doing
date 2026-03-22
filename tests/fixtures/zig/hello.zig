const std = @import("std");

pub fn add(a: i32, b: i32) i32 {
    return a + b;
}

pub fn greet(name: []const u8) void {
    std.debug.print("Hello, {s}!\n", .{name});
}
