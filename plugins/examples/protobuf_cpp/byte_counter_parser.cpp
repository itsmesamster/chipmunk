#include "parse.h"
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
    parse_string_t log_msg;
    parse_string_dup(&log_msg, "C++ Byte Counter Plugin initialized.");
    chipmunk_shared_logging_log(CHIPMUNK_SHARED_LOGGING_LEVEL_INFO, &log_msg);
    parse_string_free(&log_msg);

    parse_string_t init_detail_msg;
    char init_detail_buffer[200];
    snprintf(init_detail_buffer, sizeof(init_detail_buffer),
             "Plugin initialized with log level: %u. No specific plugin configurations handled.",
             (unsigned int)general_configs->log_level);
    parse_string_dup(&init_detail_msg, init_detail_buffer);
    chipmunk_shared_logging_log(CHIPMUNK_SHARED_LOGGING_LEVEL_DEBUG, &init_detail_msg);
    parse_string_free(&init_detail_msg);

    return true;
}

bool exports_chipmunk_parser_parser_parse(
    parse_list_u8_t *data,
    uint64_t *maybe_timestamp,
    exports_chipmunk_parser_parser_list_parse_return_t *ret,
    exports_chipmunk_parser_parser_parse_error_t *err)
{
    parse_string_t parse_entry_msg;
    parse_string_dup(&parse_entry_msg, "C++ Byte Counter Plugin: parse function entered.");
    chipmunk_shared_logging_log(CHIPMUNK_SHARED_LOGGING_LEVEL_DEBUG, &parse_entry_msg);
    parse_string_free(&parse_entry_msg);

    uint64_t byte_count = data->len;

    char log_buffer[100];
    snprintf(log_buffer, sizeof(log_buffer), "C++ Byte Counter Plugin: Parsed %llu bytes.", (unsigned long long)byte_count);
    parse_string_t log_msg;
    parse_string_dup(&log_msg, log_buffer);
    chipmunk_shared_logging_log(CHIPMUNK_SHARED_LOGGING_LEVEL_INFO, &log_msg);
    parse_string_free(&log_msg);

    ret->ptr = (exports_chipmunk_parser_parser_parse_return_t*)cabi_realloc(
        nullptr, 0, alignof(exports_chipmunk_parser_parser_parse_return_t), sizeof(exports_chipmunk_parser_parser_parse_return_t));
    ret->len = 1;

    if (ret->ptr == nullptr) {
        parse_string_t error_msg;
        parse_string_dup(&error_msg, "Failed to allocate memory for parse return item.");
        err->tag = CHIPMUNK_PARSER_PARSE_TYPES_PARSE_ERROR_UNRECOVERABLE;
        err->val.unrecoverable = error_msg;
        chipmunk_shared_logging_log(CHIPMUNK_SHARED_LOGGING_LEVEL_ERROR, &error_msg);
        parse_string_free(&error_msg);
        return false;
    }

    exports_chipmunk_parser_parser_parse_return_t& return_item = ret->ptr[0];
    return_item.consumed = byte_count;

    return_item.value.is_some = true;
    return_item.value.val.tag = CHIPMUNK_PARSER_PARSE_TYPES_PARSE_YIELD_MESSAGE;

    chipmunk_parser_parse_types_parsed_message_t& parsed_message = return_item.value.val.val.message;
    parsed_message.tag = CHIPMUNK_PARSER_PARSE_TYPES_PARSED_MESSAGE_LINE;

    char output_message_buffer[150];
    snprintf(output_message_buffer, sizeof(output_message_buffer), "Bytes processed: %llu", (unsigned long long)byte_count);
    parse_string_dup(&parsed_message.val.line, output_message_buffer);

    return true;
}

}
