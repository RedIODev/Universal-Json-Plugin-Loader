#ifndef FT_LIST_H
#define FT_LIST_H

#define CREATE_LIST_TYPE_HEADER(TYPE) \
typedef void (*TYPE##ListDeallocFP)(TYPE *, u32);\
typedef struct { \
    TYPE##ListDeallocFP dealloc_fn;\
    TYPE *data;\
    u32 length; \
} List_##TYPE; \
\
bool isValidList##TYPE(const List_##TYPE *);\
\
List_##TYPE createList##TYPE(TYPE *data, u32 length, TYPE##ListDeallocFP dealloc_fn);\
\
void destroyList##TYPE(List_##TYPE *list);\
\
TYPE *getList##TYPE(List_##TYPE *list, u32 index);\
\
List_##TYPE emptyList##TYPE();

#define CREATE_LIST_TYPE_IMPL(TYPE) \
void TYPE##no_op(TYPE *, u32) {} \
\
bool isValidList##TYPE(const List_##TYPE *list) { \
    if (list == NULL) { \
        return false; \
    } \
    return list->data != NULL && list->length != 0; \
} \
\
List_##TYPE createList##TYPE(TYPE *data, u32 length, TYPE##ListDeallocFP dealloc_fn) {\
    if (data == NULL) { \
        return (List_##TYPE) {data: NULL, length: 1, dealloc_fn: TYPE##no_op}; \
    } \
    return (List_##TYPE) {data: data, length: length, dealloc_fn: dealloc_fn }; \
}\
\
void destroyList##TYPE(List_##TYPE *list) { \
    if (!isValidList##TYPE(list)) { \
        return;\
    }\
    u32 tmp_length = list->length; \
    list->length = 0; \
    TYPE *tmp_data = list->data; \
    TYPE##ListDeallocFP tmp_fn = list->dealloc_fn; \
    list->data = NULL; \
    list->dealloc_fn = NULL; \
    if (tmp_fn == NULL) { \
        return; \
    } \
    tmp_fn(tmp_data, tmp_length); \
} \
\
TYPE *getList##TYPE(List_##TYPE *list, u32 index) { \
    if (!isValidList##TYPE(list)) { \
        return NULL; \
    } \
\
    if (list->length >= index) { \
        return NULL; \
    } \
\
    return &list->data[index]; \
} \
\
List_##TYPE emptyList##TYPE() {\
    return createList##TYPE(NULL, 0, NULL); \
}

#endif