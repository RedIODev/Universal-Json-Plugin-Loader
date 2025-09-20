
#include "header/ft_string.h"
#include <stddef.h>
#include <string.h>
#include <assert.h>

#define STRING_CAST *(String*) &

void no_op(const c8 *ignored) {}

typedef struct {
    StringDealloc dealloc_fn;
    const c8 *data;
    usize length;
 }String_impl;

static_assert(sizeof(String_impl) == sizeof(String), "Unexpected Size of hidden type String. Compiling with mismatching sizes causes UB.");

String createString(const c8 *data, usize length, StringDealloc deallocator) {
    if (data == NULL) {
        return STRING_CAST (String_impl) {data: NULL, length: 0, dealloc_fn: no_op};
    }

    return STRING_CAST (String_impl) {data: data, length: length, dealloc_fn: deallocator};
}

void destroyString(String *string) {
    if (!isValidString(string)) {
        return;
    }
    String_impl* str = (String_impl*) string;
    str->length = 0;
    const c8 *tmp_data = str->data;
    StringDealloc tmp_fn = str->dealloc_fn;
    str->data = NULL;
    str->dealloc_fn = NULL;
    if (tmp_fn == NULL) {
        return;
    }
    tmp_fn(tmp_data);
}

bool isValidString(const String *string) {
    if (string == NULL) {
        return false;
    }
    String_impl* str = (String_impl*) string;
    return str->data != NULL;
}

c8 getCharString(const String *string, usize index) {
    if (!isValidString(string)) {
        return 0;
    }
    String_impl* str = (String_impl*) string;

    if (str->length >= index) {
        return 0;
    }

    return str->data[index];
}

usize getLengthString(const String *string) {
    String_impl* str = (String_impl*) string;
    return str->length;
}

const c8 *getViewString(const String *string, usize start, usize end) {
    if (!isValidString(string)) {
        return NULL;
    }
    String_impl* str = (String_impl*) string;
    if (start >= end || start >= str->length || end > str->length) {
        return NULL;
    }
    return str->data + start;

}

