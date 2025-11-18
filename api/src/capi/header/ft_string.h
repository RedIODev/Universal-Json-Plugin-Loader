#ifndef FT_STRING_H
#define FT_STRING_H

#include "ft_types.h"
#include "ft_list.h"
#include "ft_util.h"



typedef void (*StringDealloc)(const c8 *, usize);

// Immutable owning String type with variable allocator and fixed length.
// Takes ownership of the passed buffer.
// Rather the String instance is valid can be tested with the isValidString(String*) function.
// All operations on invalid Strings are nops.
// A String instance with a null deallocate function is valid and it is assumed that the buffer has a lifetime greater than the String instance.
typedef struct {
    u8 internal[sizeof(StringDealloc) + sizeof(c8*) + sizeof(usize)];
} String;

// Creates a new String instance from a buffer (taking ownership), the size of the buffer and a function to deallocate the buffer.
// When a String instance is created with invalid arguments the ownership of the buffer is returned to the caller.
// The deallocator can be null in which case the buffer will be leaked if not managed otherwise.
String createString(const c8 *, usize, StringDealloc);

// Creates an invalid String containing the ServiceError as an Invalid Value. It does not allocate anything, just storing the error in the String handle.
String fromErrorString(ServiceError);

// Attempts to reinterpret an invalid String as an Service Error. Returns null if not an ServiceError. 
ServiceError *asErrorString(const String *);

// Destroys the String including it's content. The instance is no longer valid after a call to this function.
// Calling destroy on invalid Strings has no effects and is considered a nop (just like any other function).
void destroyString(String *);

// True if the String is valid and contains owned data.
bool isValidString(const String *);

// Gets a character at a given index from a given String.
c8 getCharString(const String *, usize);

usize getLengthString(const String *);

// Gets a view from a given String, start index (inclusive) and end index (exclusive). 
// The view is NOT null terminated.
const c8 *getViewString(const String *, usize, usize);

CREATE_LIST_TYPE_HEADER(String)

#endif