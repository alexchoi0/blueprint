"""Sorting algorithms in Starlark."""

def quicksort(arr):
    """Quicksort implementation."""
    if len(arr) <= 1:
        return arr
    pivot = arr[len(arr) // 2]
    left = [x for x in arr if x < pivot]
    middle = [x for x in arr if x == pivot]
    right = [x for x in arr if x > pivot]
    return quicksort(left) + middle + quicksort(right)

def mergesort(arr):
    """Mergesort implementation."""
    if len(arr) <= 1:
        return arr
    mid = len(arr) // 2
    left = mergesort(arr[:mid])
    right = mergesort(arr[mid:])
    return merge(left, right, 0, 0, [])

def merge(left, right, i, j, result):
    """Merge two sorted lists recursively."""
    if i >= len(left):
        return result + right[j:]
    if j >= len(right):
        return result + left[i:]
    if left[i] <= right[j]:
        return merge(left, right, i + 1, j, result + [left[i]])
    else:
        return merge(left, right, i, j + 1, result + [right[j]])

def insertion_sort(arr):
    """Insertion sort implementation using recursion."""
    if len(arr) <= 1:
        return arr
    return insert(insertion_sort(arr[:-1]), arr[-1])

def insert(sorted_arr, elem):
    """Insert element into sorted array."""
    if len(sorted_arr) == 0:
        return [elem]
    if elem <= sorted_arr[0]:
        return [elem] + sorted_arr
    return [sorted_arr[0]] + insert(sorted_arr[1:], elem)

def selection_sort(arr):
    """Selection sort using recursion."""
    if len(arr) <= 1:
        return arr
    min_idx = find_min_idx(arr, 0, 0)
    min_val = arr[min_idx]
    rest = arr[:min_idx] + arr[min_idx + 1:]
    return [min_val] + selection_sort(rest)

def find_min_idx(arr, current_idx, min_idx):
    """Find index of minimum element."""
    if current_idx >= len(arr):
        return min_idx
    if arr[current_idx] < arr[min_idx]:
        return find_min_idx(arr, current_idx + 1, current_idx)
    return find_min_idx(arr, current_idx + 1, min_idx)

data = [64, 34, 25, 12, 22, 11, 90, 5, 77, 30]

print("Original:", data)
print("Quicksort:", quicksort(data))
print("Mergesort:", mergesort(data))
print("Insertion sort:", insertion_sort(data))
print("Selection sort:", selection_sort(data))
