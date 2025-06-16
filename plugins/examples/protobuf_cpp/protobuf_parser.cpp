#include "parse.h"
#include "test/test.pb.h"
#include "pb_decode.h"

#include <iostream>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

extern "C" __attribute__((export_name("cabi_realloc")))
void* cabi_realloc(void* ptr, size_t old_size, size_t align, size_t new_size) {
    if (ptr == nullptr) {
        return aligned_alloc(align, new_size);
    }
    if (new_size == 0) {
        free(ptr);
        return nullptr;
    }
    void* new_ptr = aligned_alloc(align, new_size);
    if (new_ptr && ptr) {
        memcpy(new_ptr, ptr, old_size < new_size ? old_size : new_size);
        free(ptr);
    }
    return new_ptr;
}

extern "C" {

void exports_chipmunk_parser_parser_get_version(exports_chipmunk_parser_parser_version_t *ret) {
    ret->major = 0;
    ret->minor = 1;
    ret->patch = 0;
}

void exports_chipmunk_parser_parser_get_config_schemas(exports_chipmunk_parser_parser_list_config_schema_item_t *ret) {
    ret->ptr = NULL;
    ret->len = 0;
}

void exports_chipmunk_parser_parser_get_render_options(exports_chipmunk_parser_parser_render_options_t *ret) {
    ret->columns_options.is_some = false;
}

bool exports_chipmunk_parser_parser_init(
    exports_chipmunk_parser_parser_parser_config_t *general_configs,
    exports_chipmunk_parser_parser_list_config_item_t *plugin_configs,
    exports_chipmunk_parser_parser_init_error_t *err)
{
    parse_string_t msg;
    parse_string_dup(&msg, "Nanopb-based Protobuf Plugin Initialized");
    chipmunk_shared_logging_log(CHIPMUNK_SHARED_LOGGING_LEVEL_INFO, &msg);
    parse_string_free(&msg);
    return true;
}

bool exports_chipmunk_parser_parser_parse(
    parse_list_u8_t *data,
    uint64_t *maybe_timestamp,
    exports_chipmunk_parser_parser_list_parse_return_t *ret,
    exports_chipmunk_parser_parser_parse_error_t *err)
{
    parse_string_t msg;
    parse_string_dup(&msg, "Entering Nanopb Parser...");
    chipmunk_shared_logging_log(CHIPMUNK_SHARED_LOGGING_LEVEL_DEBUG, &msg);
    parse_string_free(&msg);

    my_package_Request request = my_package_Request_init_zero;
    pb_istream_t stream = pb_istream_from_buffer(data->ptr, data->len);

    bool decode_success = pb_decode(&stream, my_package_Request_fields, &request);

    char log_buf[256];

    if (!decode_success) {
        snprintf(log_buf, sizeof(log_buf), "Failed to decode with Nanopb. Consumed: %llu bytes.", (unsigned long long)data->len);
        parse_string_t err_msg;
        parse_string_dup(&err_msg, log_buf);
        chipmunk_shared_logging_log(CHIPMUNK_SHARED_LOGGING_LEVEL_ERROR, &err_msg);
        parse_string_free(&err_msg);

        goto return_empty;
    }

    snprintf(log_buf, sizeof(log_buf),
        "Nanopb Parsed: ID=%llu, Str='%s', TM=%llu",
        (unsigned long long)request.id,
        (char*)request.str.arg,
        (unsigned long long)request.tm);


    parse_string_dup(&msg, log_buf);
    chipmunk_shared_logging_log(CHIPMUNK_SHARED_LOGGING_LEVEL_INFO, &msg);
    parse_string_free(&msg);

return_empty:

    ret->ptr = (exports_chipmunk_parser_parser_parse_return_t*)cabi_realloc(
        nullptr, 0, alignof(exports_chipmunk_parser_parser_parse_return_t), sizeof(exports_chipmunk_parser_parser_parse_return_t));
    ret->len = 1;

    if (!ret->ptr) {
        parse_string_dup(&msg, "Memory allocation failed for parse return.");
        err->tag = CHIPMUNK_PARSER_PARSE_TYPES_PARSE_ERROR_UNRECOVERABLE;
        err->val.unrecoverable = msg;
        return false;
    }

    auto& item = ret->ptr[0];
    item.consumed = data->len;
    item.value.is_some = true;
    item.value.val.tag = CHIPMUNK_PARSER_PARSE_TYPES_PARSE_YIELD_MESSAGE;
    item.value.val.val.message.tag = CHIPMUNK_PARSER_PARSE_TYPES_PARSED_MESSAGE_LINE;

    snprintf(log_buf, sizeof(log_buf),
        "PROTOBUF_MSG (Nanopb): ID=%llu | TM=%llu | Str=%s",
        (unsigned long long)request.id,
        (unsigned long long)request.tm,
        (char*)request.str.arg); // Assuming str is a callback string with the actual string in arg

    parse_string_dup(&item.value.val.val.message.val.line, log_buf);
    return true;
}

}
