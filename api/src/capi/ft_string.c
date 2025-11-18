
#include "header/ft_string.h"
#include "header/ft_list.h"
#include <stddef.h>
#include <string.h>
#include <assert.h>

// #define STRING_ERROR_FLAG 0xEEEEEEEE

// #define STRING_CAST *(String*) &

#define FLAG_SIZE sizeof(StringDealloc) + sizeof(c8*)

CREATE_LIST_TYPE_IMPL(String)

void no_op(const u8 *, usize) {}

typedef struct {
    usize length;
    const c8 *data;
    StringDealloc dealloc_fn;
 }StringValid;

 typedef struct {
    ServiceError error;
    u8 flag[FLAG_SIZE];
 }StringError;

 typedef union {
    String opaque;
    StringValid valid;
    StringError error;
 } StringUnion;

String castOpaque(StringUnion strUnion) {
    return strUnion.opaque;
}

StringValid *castValid(StringUnion *strUnion) {
    if (strUnion == NULL || strUnion->valid.data == NULL) {
        return NULL;
    }
    return &strUnion->valid;
}

StringError *castError(StringUnion *strUnion) {
    if (strUnion == NULL) {
        return NULL;
    }
    for (u32 i = 0; i <FLAG_SIZE; i++) {
        if (strUnion->error.flag[i]!= 0) {
            return NULL;
        }
    }
    return &strUnion->error;
}

static_assert(sizeof(StringUnion) <= sizeof(String), "Not all String variants fit into the opaque type");

// static_assert(sizeof(StringValid) == sizeof(String), "Unexpected Size of hidden type String. Compiling with mismatching sizes causes UB.");
// static_assert(sizeof(usize) >= sizeof(ServiceError), "ServiceError doesn't fit inside Strings Handle.");

String createString(const c8 *data, usize length, StringDealloc deallocator) {
    if (data == NULL) {
        return castOpaque((StringUnion){valid: {data: NULL, length: 0, dealloc_fn: no_op}});
    }
    return castOpaque((StringUnion) {valid: {data: data, length: length, dealloc_fn: deallocator}});
}

String fromErrorString(ServiceError serviceError) {
    return castOpaque((StringUnion) {error: {flag: {0}, error: serviceError}});
}

ServiceError *asErrorString(const String *string) {
    StringError *error = castError((StringUnion *)string);
    if (error == NULL) {
        return NULL;
    }
    return &error->error;
}

void destroyString(String *string) {
    StringValid *str = castValid((StringUnion *)string);
    if (str == NULL) {
        return;
    }
    usize tmp_length = str->length;
    str->length = 0;
    const c8 *tmp_data = str->data;
    StringDealloc tmp_fn = str->dealloc_fn;
    str->data = NULL;
    str->dealloc_fn = NULL;
    if (tmp_fn == NULL) {
        return;
    }
    tmp_fn(tmp_data, tmp_length);
}

bool isValidString(const String *string) {
    return castValid((StringUnion *)string) != NULL;
}

c8 getCharString(const String *string, usize index) {
    StringValid *str = castValid((StringUnion *)string);
    if (str == NULL) {
        return 0;
    }
    if (str->length >= index) {
        return 0;
    }

    return str->data[index];
}

usize getLengthString(const String *string) {
    StringValid *str = castValid((StringUnion *)string);
    if (str == NULL) {
        return 0;
    }
    return str->length;
}

const c8 *getViewString(const String *string, usize start, usize end) {
    StringValid *str = castValid((StringUnion *)string);
    if (str == NULL) {
        return NULL;
    }
    if (start >= end || start >= str->length || end > str->length) {
        return NULL;
    }
    return str->data + start;

}

