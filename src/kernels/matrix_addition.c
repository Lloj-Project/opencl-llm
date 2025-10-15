// add two matrices
__kernel void matrix_addition(
    __global double* matrix_a,
    __global double* matrix_b,
    __global double* result,
    const int width_a,
    const int height_a,
    const int width_b,
    const int height_b,
){
    row_index = get_global_id(0);
    col_index = get_global_id(1);
    size_a = width_a * height_a;
    size_b = width_b * height_b;
    index_a = row_index * width_a + col_index;
    index_b = row_index * width_a + col_index;
    index_res = size_a > size_b ? index_a : index_b;
    //handle when they are not the same size
    if (index_a > size_a) {
        a_val = 0;
    } 
    else {
        a_val = matrix_a[index_a];
    }

    if (index_b > size_b) {
        b_val = 0;
    } 
    else {
        b_val = matrix_b[index_b];
    }
    result[index_res] = matrix_a[index_a] + matrix_b[index_b]
}
