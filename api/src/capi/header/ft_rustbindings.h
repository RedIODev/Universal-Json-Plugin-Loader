#ifndef FT_RUSTBINDINGS_H
#define FT_RUSTBINDINGS_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define _STDINT_H 1

#define _FEATURES_H 1

#define _DEFAULT_SOURCE 1

#define __GLIBC_USE_ISOC2Y 0

#define __GLIBC_USE_ISOC23 0

#define __USE_ISOC11 1

#define __USE_ISOC99 1

#define __USE_ISOC95 1

#define __USE_POSIX_IMPLICITLY 1

#define _POSIX_SOURCE 1

#define _POSIX_C_SOURCE 200809

#define __USE_POSIX 1

#define __USE_POSIX2 1

#define __USE_POSIX199309 1

#define __USE_POSIX199506 1

#define __USE_XOPEN2K 1

#define __USE_XOPEN2K8 1

#define _ATFILE_SOURCE 1

#define __WORDSIZE 64

#define __WORDSIZE_TIME64_COMPAT32 1

#define __SYSCALL_WORDSIZE 64

#define __TIMESIZE 64

#define __USE_TIME_BITS64 1

#define __USE_MISC 1

#define __USE_ATFILE 1

#define __USE_FORTIFY_LEVEL 0

#define __GLIBC_USE_DEPRECATED_GETS 0

#define __GLIBC_USE_DEPRECATED_SCANF 0

#define __GLIBC_USE_C23_STRTOL 0

#define _STDC_PREDEF_H 1

#define __STDC_IEC_559__ 1

#define __STDC_IEC_60559_BFP__ 201404

#define __STDC_IEC_559_COMPLEX__ 1

#define __STDC_IEC_60559_COMPLEX__ 201404

#define __STDC_ISO_10646__ 201706

#define __GNU_LIBRARY__ 6

#define __GLIBC__ 2

#define __GLIBC_MINOR__ 41

#define _SYS_CDEFS_H 1

#define __glibc_c99_flexarr_available 1

#define __LDOUBLE_REDIRECTS_TO_FLOAT128_ABI 0

#define __HAVE_GENERIC_SELECTION 1

#define __GLIBC_USE_LIB_EXT2 0

#define __GLIBC_USE_IEC_60559_BFP_EXT 0

#define __GLIBC_USE_IEC_60559_BFP_EXT_C23 0

#define __GLIBC_USE_IEC_60559_EXT 0

#define __GLIBC_USE_IEC_60559_FUNCS_EXT 0

#define __GLIBC_USE_IEC_60559_FUNCS_EXT_C23 0

#define __GLIBC_USE_IEC_60559_TYPES_EXT 0

#define _BITS_TYPES_H 1

#define _BITS_TYPESIZES_H 1

#define __OFF_T_MATCHES_OFF64_T 1

#define __INO_T_MATCHES_INO64_T 1

#define __RLIM_T_MATCHES_RLIM64_T 1

#define __STATFS_MATCHES_STATFS64 1

#define __KERNEL_OLD_TIMEVAL_MATCHES_TIMEVAL64 1

#define __FD_SETSIZE 1024

#define _BITS_TIME64_H 1

#define _BITS_WCHAR_H 1

#define _BITS_STDINT_INTN_H 1

#define _BITS_STDINT_UINTN_H 1

#define _BITS_STDINT_LEAST_H 1

#define INT8_MIN -128

#define INT16_MIN -32768

#define INT32_MIN -2147483648

#define INT8_MAX 127

#define INT16_MAX 32767

#define INT32_MAX 2147483647

#define UINT8_MAX 255

#define UINT16_MAX 65535

#define UINT32_MAX 4294967295

#define INT_LEAST8_MIN -128

#define INT_LEAST16_MIN -32768

#define INT_LEAST32_MIN -2147483648

#define INT_LEAST8_MAX 127

#define INT_LEAST16_MAX 32767

#define INT_LEAST32_MAX 2147483647

#define UINT_LEAST8_MAX 255

#define UINT_LEAST16_MAX 65535

#define UINT_LEAST32_MAX 4294967295

#define INT_FAST8_MIN -128

#define INT_FAST16_MIN -9223372036854775808ull

#define INT_FAST32_MIN -9223372036854775808ull

#define INT_FAST8_MAX 127

#define INT_FAST16_MAX 9223372036854775807

#define INT_FAST32_MAX 9223372036854775807

#define UINT_FAST8_MAX 255

#define UINT_FAST16_MAX -1

#define UINT_FAST32_MAX -1

#define INTPTR_MIN -9223372036854775808ull

#define INTPTR_MAX 9223372036854775807

#define UINTPTR_MAX -1

#define PTRDIFF_MIN -9223372036854775808ull

#define PTRDIFF_MAX 9223372036854775807

#define SIG_ATOMIC_MIN -2147483648

#define SIG_ATOMIC_MAX 2147483647

#define SIZE_MAX -1

#define WINT_MIN 0

#define WINT_MAX 4294967295

#define _UCHAR_H 1

#define __mbstate_t_defined 1

#define ____mbstate_t_defined 1

#define __bool_true_false_are_defined 1

#define true_ 1

#define false_ 0

enum CServiceError {
  Success = 0,
  CoreInternalError = 1,
  InvalidInput0 = 2,
  InvalidInput1 = 3,
  InvalidInput2 = 4,
  InvalidInput3 = 5,
  InvalidInput4 = 6,
  InvalidInput5 = 7,
  InvalidInput6 = 8,
  InvalidInput7 = 9,
  NotFound = 10,
  Unauthorized = 11,
  Duplicate = 12,
  PluginUninit = 13,
  InvalidResponse = 14,
  ShutingDown = 15,
};
typedef uint32_t CServiceError;

typedef unsigned short C__uint16_t;

typedef C__uint16_t C__uint_least16_t;

typedef C__uint_least16_t Cchar16_t;

typedef struct __BindgenUnionField_c_uint {

} __BindgenUnionField_c_uint;

typedef struct __BindgenUnionField__________c_char__________4 {

} __BindgenUnionField__________c_char__________4;

typedef struct C__mbstate_t__bindgen_ty_1 {
  struct __BindgenUnionField_c_uint __wch;
  struct __BindgenUnionField__________c_char__________4 __wchb;
  uint32_t bindgen_union_field;
} C__mbstate_t__bindgen_ty_1;

typedef struct C__mbstate_t {
  int __count;
  struct C__mbstate_t__bindgen_ty_1 __value;
} C__mbstate_t;

typedef struct C__mbstate_t Cmbstate_t;

typedef unsigned int C__uint32_t;

typedef C__uint32_t C__uint_least32_t;

typedef C__uint_least32_t Cchar32_t;

typedef uint8_t Cu8;

typedef struct CString {
  Cu8 internal[24];
} CString;

typedef unsigned char Cc8;

typedef uintptr_t Cusize;

typedef void (*CStringDealloc)(const Cc8 *arg1, Cusize arg2);

typedef uint32_t Cu32;

typedef void (*CStringListDeallocFP)(struct CString *arg1, Cu32 arg2);

typedef struct CList_String {
  CStringListDeallocFP dealloc_fn;
  struct CString *data;
  Cu32 length;
} CList_String;

typedef uint64_t Cu64;

typedef struct CUuid {
  Cu64 higher;
  Cu64 lower;
} CUuid;

typedef struct CEventHandler {
  CEventHandlerFP function;
  struct CUuid handler_id;
  CServiceError error;
} CEventHandler;

typedef struct CEventHandler (*CHandlerRegisterService)(CEventHandlerFP arg1,
                                                        struct CUuid arg2,
                                                        struct CString arg3);

typedef CServiceError (*CHandlerUnregisterService)(struct CUuid arg1,
                                                   struct CUuid arg2,
                                                   struct CString arg3);

typedef CServiceError (*CEventRegisterService)(struct CString arg1,
                                               struct CUuid arg2,
                                               struct CString arg3);

typedef CServiceError (*CEventUnregisterService)(struct CUuid arg1, struct CString arg2);

typedef CServiceError (*CEventTriggerService)(struct CUuid arg1,
                                              struct CString arg2,
                                              struct CString arg3);

typedef struct CEndpointResponse {
  struct CString response;
  CServiceError error;
} CEndpointResponse;

typedef struct CEndpointResponse (*CRequestHandlerFP)(CContextSupplier arg1, struct CString arg2);

typedef CServiceError (*CEndpointRegisterService)(struct CString arg1,
                                                  struct CString arg2,
                                                  struct CUuid arg3,
                                                  struct CString arg4,
                                                  CRequestHandlerFP arg5);

typedef CServiceError (*CEndpointUnregisterService)(struct CUuid arg1, struct CString arg2);

typedef struct CEndpointResponse (*CEndpointRequestService)(struct CString arg1, struct CString arg2);

typedef struct CApplicationContext {
  CHandlerRegisterService handlerRegisterService;
  CHandlerUnregisterService handlerUnregisterService;
  CEventRegisterService eventRegisterService;
  CEventUnregisterService eventUnregisterService;
  CEventTriggerService eventTriggerService;
  CEndpointRegisterService endpointRegisterService;
  CEndpointUnregisterService endpointUnregisterService;
  CEndpointRequestService endpointRequestService;
} CApplicationContext;

typedef struct CApplicationContext (*CContextSupplier)(void);

typedef void (*CEventHandlerFP)(CContextSupplier arg1, struct CString arg2);

typedef uint16_t Cu16;

typedef struct CApiVersion {
  Cu16 major;
  Cu8 feature;
  Cu8 patch;
} CApiVersion;

extern const struct CApiVersion API_VERSION;

extern uintptr_t mbrtoc16(Cchar16_t *__pc16, const char *__s, uintptr_t __n, Cmbstate_t *__p);

extern uintptr_t c16rtomb(char *__s, Cchar16_t __c16, Cmbstate_t *__ps);

extern uintptr_t mbrtoc32(Cchar32_t *__pc32, const char *__s, uintptr_t __n, Cmbstate_t *__p);

extern uintptr_t c32rtomb(char *__s, Cchar32_t __c32, Cmbstate_t *__ps);

extern struct CString createString(const Cc8 *arg1, Cusize arg2, CStringDealloc arg3);

extern void destroyString(struct CString *arg1);

extern bool isValidString(const struct CString *arg1);

extern Cc8 getCharString(const struct CString *arg1, Cusize arg2);

extern Cusize getLengthString(const struct CString *arg1);

extern const Cc8 *getViewString(const struct CString *arg1, Cusize arg2, Cusize arg3);

extern bool isValidListString(const struct CList_String *arg1);

extern struct CList_String createListString(struct CString *data,
                                            Cu32 length,
                                            CStringListDeallocFP dealloc_fn);

extern void destroyListString(struct CList_String *list);

extern struct CString *getListString(struct CList_String *list, Cu32 index);

extern struct CList_String emptyListString(void);

extern CEventHandlerFP pluginMain(struct CUuid arg1);

#endif  /* FT_RUSTBINDINGS_H */
