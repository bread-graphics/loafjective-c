//              Copyright John Nunley 2022
// Distributed under the Boost Software License, Version 1.0.
//       (See accompanying file LICENSE or copy at
//         https://www.boost.org/LICENSE_1_0.txt)

#include <objc/objc.h>

// Try to run the given function, and catch an error if it fails.
// 
// As the error handling ABI varies from platform to platform, and is
// not stable, this is unfortunately the only way to handle errors.
int loafTryRunAndCatch(
    void (*function)(void *),
    void *closure,
    id *error
) {
    @try {
        function(closure);
        return 0;
    } @catch (id e) {
        *error = [e retain];
        return 1;
    }
}