
#include "header/ft_string.h"
#include "header/ft_list.h"
#include <stddef.h>
#include <string.h>
#include <assert.h>

#define FLAG_SIZE sizeof(StringDealloc) + sizeof(c8*)

CREATE_LIST_TYPE_IMPL(String)

void no_op(const u8 *, usize) {}

typedef struct {
    usize length;
    const c8 *data;
    StringDealloc dealloc_fn;
 }StringData;

 typedef struct {
    ServiceError error;
    u8 flag[FLAG_SIZE];
 }StringError;

 typedef union {
    String opaque;
    StringData valid;
    StringError error;
 } StringUnion;

String castOpaque(StringUnion strUnion) {
    return strUnion.opaque;
}

StringData *castData(StringUnion *strUnion) {
    if (strUnion == NULL || strUnion->valid.data == NULL) {
        return NULL;
    }
    return &strUnion->valid;
}

StringError *castError(StringUnion *strUnion) {
    NULL_GUARD(strUnion, NULL)
    
    for (u32 i = 0; i <FLAG_SIZE; i++) {
        if (strUnion->error.flag[i]!= 0) {
            return NULL;
        }
    }
    //invalid because the error variant schuld only be active on error not success. (A SUCCESS here signalizes an 0 initialzed String which is not a valid error.)
    if (strUnion->error.error == SERVICE_SUCCESS) { 
        return NULL;
    }
    return &strUnion->error;
}

static_assert(sizeof(StringUnion) <= sizeof(String), "Not all String variants fit into the opaque type");

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
    NULL_GUARD(error, NULL)
    return &error->error;
}

void destroyString(String *string) {
    StringData *str = castData((StringUnion *)string);
    NULL_GUARD(str,)
    
    usize tmp_length = str->length;
    str->length = 0;
    const c8 *tmp_data = str->data;
    StringDealloc tmp_fn = str->dealloc_fn;
    str->data = NULL;
    str->dealloc_fn = NULL;
    NULL_GUARD(tmp_fn,)

    tmp_fn(tmp_data, tmp_length);
}

bool isValidString(const String *string) {
    return castData((StringUnion *)string) != NULL;
}

c8 getCharString(const String *string, usize index) {
    StringData *str = castData((StringUnion *)string);
    NULL_GUARD(str, 0)
    if (str->length >= index) {
        return 0;
    }

    return str->data[index];
}

usize getLengthString(const String *string) {
    StringData *str = castData((StringUnion *)string);
    NULL_GUARD(str, 0)
    return str->length;
}

const c8 *getViewString(const String *string, usize start, usize end) {
    StringData *str = castData((StringUnion *)string);
    NULL_GUARD(str, NULL)
    if (start >= end || start >= str->length || end > str->length) {
        return NULL;
    }
    return str->data + start;

}

