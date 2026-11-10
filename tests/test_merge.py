import gc
import sys

import pytest

from json_multi_merge import merge

LEAK_ITERATIONS = 1000


def test_no_arguments():
    assert merge() == {}


def test_single_argument():
    assert merge({"a": 1}) == {"a": 1}


def test_basic_merge():
    assert merge({"a": 1, "b": 2}, {"b": 3, "c": 4}) == {"a": 1, "b": 3, "c": 4}


def test_variadic_merge():
    assert merge({"a": 1}, {"b": 2}, {"c": 3}) == {"a": 1, "b": 2, "c": 3}


def test_nested_merge():
    base = {"user": {"name": "Alice", "contact": {"email": "alice@example.com"}}}
    update = {"user": {"contact": {"phone": "555-1234"}, "preferences": {"theme": "dark"}}}
    assert merge(base, update) == {
        "user": {
            "name": "Alice",
            "contact": {"email": "alice@example.com", "phone": "555-1234"},
            "preferences": {"theme": "dark"},
        }
    }


def test_lists_are_replaced():
    assert merge({"tags": ["old"], "items": [1, 2]}, {"tags": ["new"], "items!": [3, 4]}) == {
        "tags": ["new"],
        "items": [3, 4],
    }


def test_omit_modifier():
    assert merge({"keep": 1, "remove_me": 2}, {"remove_me--": 3, "new_key": 4}) == {
        "keep": 1,
        "new_key": 4,
    }


def test_replace_modifier():
    base = {"config": {"debug": False, "plugins": ["basic"]}}
    update = {"config!": {"production": True, "plugins": ["advanced"]}}
    assert merge(base, update) == {"config": {"production": True, "plugins": ["advanced"]}}


def test_nested_modifiers():
    base = {"system": {"settings": {"logging": {"level": "info"}}}}
    update = {"system!": {"settings--": {}, "new_settings": {"cache_size": "256MB"}}}
    assert merge(base, update) == {"system": {"new_settings": {"cache_size": "256MB"}}}


def test_history_appends_lists():
    assert merge({"logs-history": [1, 2]}, {"logs-history": [3, 4]}) == {
        "logs-history": [1, 2, 3, 4]
    }


def test_history_with_non_list_replaces():
    assert merge({"logs-history": [1, 2]}, {"logs-history": "not a list"}) == {
        "logs-history": "not a list"
    }


def test_value_types_round_trip():
    value = {"s": "x", "i": -7, "big": 2**62, "f": 1.5, "t": True, "f2": False, "n": None}
    result = merge(value)
    assert result == value
    assert type(result["t"]) is bool
    assert type(result["i"]) is int
    assert type(result["f"]) is float


def test_type_conflict_last_wins():
    assert merge({"data": {"values": [1, 2, 3]}}, {"data": "invalid"}) == {"data": "invalid"}


def test_inputs_are_not_mutated():
    a = {"a": {"x": 1}}
    b = {"a": {"y": 2}, "b--": None}
    merge(a, b)
    assert a == {"a": {"x": 1}}
    assert b == {"a": {"y": 2}, "b--": None}


@pytest.mark.parametrize("bad", [[1, 2, 3], "not a dict", 1, None])
def test_non_dict_input_raises_type_error(bad):
    with pytest.raises(TypeError, match="All inputs must be dictionaries"):
        merge({"a": 1}, bad)


def test_unsupported_value_type_raises_type_error():
    with pytest.raises(TypeError, match="Unsupported type"):
        merge({"a": object()})


def test_non_string_key_raises_type_error():
    with pytest.raises(TypeError):
        merge({1: "a"})


def _marked_dicts(marker):
    # Each dict holds a list so the garbage collector keeps tracking it.
    return [
        obj for obj in gc.get_objects()
        if type(obj) is dict and obj.get("leak_marker") == marker
    ]


@pytest.mark.parametrize("arg_count", [1, 2, 3])
def test_merge_does_not_leak_dicts(arg_count):
    marker = [f"leak-check-{arg_count}"]

    gc.collect()
    for _ in range(LEAK_ITERATIONS):
        merge(*[{"leak_marker": marker, "n": [i]} for i in range(arg_count)])
    gc.collect()
    leaked = len(_marked_dicts(marker))

    assert leaked == 0, f"{leaked} dicts leaked over {LEAK_ITERATIONS} calls"


@pytest.mark.parametrize("arg_count", [1, 2, 3])
def test_merge_does_not_hold_input_references(arg_count):
    inputs = [{"nested": {"n": [i]}} for i in range(arg_count)]
    refcounts_before = [sys.getrefcount(obj) for obj in inputs]

    for _ in range(LEAK_ITERATIONS):
        merge(*inputs)

    assert [sys.getrefcount(obj) for obj in inputs] == refcounts_before
