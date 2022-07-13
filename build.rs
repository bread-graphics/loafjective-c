//              Copyright John Nunley 2022
// Distributed under the Boost Software License, Version 1.0.
//       (See accompanying file LICENSE or copy at
//         https://www.boost.org/LICENSE_1_0.txt)

fn main() {
    // compile "extern/error.m" into a static library using the cc crate
    // and link it into the executable
    cc::Build::new()
        .file("extern/error.m")
        .flag("-fobjc-exceptions")
        .flag("-w")
        .compile("error");
}
