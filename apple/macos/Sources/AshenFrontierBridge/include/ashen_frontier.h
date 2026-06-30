#ifndef ASHEN_FRONTIER_H
#define ASHEN_FRONTIER_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct AfWorld AfWorld;

typedef struct AfEntityPosition {
    uint32_t id;
    float x;
    float y;
} AfEntityPosition;

AfWorld *af_world_create(uint16_t width, uint16_t height);
void af_world_destroy(AfWorld *world);
void af_world_step(AfWorld *world, uint32_t steps);
void af_world_spawn_horde(AfWorld *world, uint32_t enemy_count);
uint64_t af_world_tick(const AfWorld *world);
size_t af_world_unit_count(const AfWorld *world);
size_t af_world_enemy_count(const AfWorld *world);
size_t af_world_read_units(
    const AfWorld *world,
    AfEntityPosition *out_positions,
    size_t capacity
);
size_t af_world_read_enemies(
    const AfWorld *world,
    AfEntityPosition *out_positions,
    size_t capacity
);

#ifdef __cplusplus
}
#endif

#endif
