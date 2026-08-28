#pragma once

#include <stddef.h>

typedef struct etna_listnode {
    struct etna_listnode* prev;
    struct etna_listnode* next;
} etna_listnode_t;

#ifndef container_of
#define etna_container_of(ptr, type, member) ((type*)((char*)(ptr) - offsetof(type, member)))
#else
#define etna_container_of container_of
#endif

static inline void etna_list_init(etna_listnode_t* head) {
    head->next = head;
    head->prev = head;
}

static inline void etna_list_add(etna_listnode_t* head, etna_listnode_t* entry) {
    entry->next = head->next;
    entry->prev = head;
    head->next->prev = entry;
    head->next = entry;
}

static inline void etna_list_del(etna_listnode_t* node) {
    etna_listnode_t* next = node->next;
    etna_listnode_t* prev = node->prev;

    prev->next = next;
    next->prev = prev;
}

#define ETNA_LIST_FOR_EACH_ENTRY(pos, head, member)                                               \
    for (pos = ((head) && (head)->next)                                                           \
                   ? etna_container_of((head)->next, __typeof__(*pos), member)                    \
                   : NULL;                                                                        \
         pos && &pos->member != (head);                                                           \
         pos = (pos->member.next) ? etna_container_of(pos->member.next, __typeof__(*pos), member) \
                                  : NULL)