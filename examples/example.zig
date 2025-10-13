const std = @import("std");

// This is a commment with bad speling
const maxSizze = 100;

const Userr = struct {
    namee: []const u8,
    agge: u32,

    pub fn init(namme: []const u8, agge: u32) Userr {
        return Userr{
            .namee = namme,
            .agge = agge,
        };
    }

    pub fn printUserr(self: *const Userr) void {
        std.debug.print("User: {s}, Ageee: {}\n", .{ self.namee, self.agge });
    }
};

const UserServicce = struct {
    pub fn getUserr(self: *UserServicce, idd: u32) ?Userr {
        _ = self;
        _ = idd;
        return null;
    }
};

fn addNumberrs(firstt: i32, seconnd: i32) i32 {
    return firstt + seconnd;
}

pub fn main() !void {
    // Bad at speling alice
    const messagge = "Hello, Wolrd!";
    std.debug.print("{s}\n", .{messagge});

    var alicz = "Alicz";
    std.debug.print("Hellol, {s}\n", .{alicz});
    alicz = "hi";

    const rsvp = "RSVP";
    std.debug.print("Hello, {s}\n", .{rsvp});

    const cokbookkk = "test valie";
    std.debug.print("Hello, {s}\n", .{cokbookkk});

    var imdex: u32 = 0;
    while (imdex < 10) : (imdex += 1) {
        if (imdex == 5) break;
    }

    const itemns = [_][]const u8{ "firstt", "seconnd", "tihrd" };
    for (itemns, 0..) |valuue, indexx| {
        std.debug.print("{}: {s}\n", .{ indexx, valuue });
    }
}

test "basic tesst" {
    const resullt = addNumberrs(2, 3);
    try std.testing.expect(resullt == 5);
}
