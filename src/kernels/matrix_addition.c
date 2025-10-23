// add two matrices
__kernel void matrix_addition(
    __global double* matrix_a,
    __global double* matrix_b,
    __global double* result,
    const int width_a,
    const int height_a,
    const int width_b,
    const int height_b
){
    int row_index = get_global_id(0);
    int col_index = get_global_id(1);
    int size_a = width_a * height_a;
    int size_b = width_b * height_b;
    int index_a = row_index * width_a + col_index;
    int index_b = row_index * width_a + col_index;
    int index_res = size_a > size_b ? index_a : index_b;
    //handle when they are not the same size
    int a_val = 0;
    int b_val = 0;
    if (index_a <= size_a) {
        a_val = matrix_a[index_a];
    } 
    if (index_b <= size_b) {
        int b_val = matrix_b[index_b];
    } 
    result[index_res] = a_val + b_val;
}
