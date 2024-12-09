SELECT
    col_a,
      col_b,
    COUNT(*)   AS   num,
    SUM(num)   OVER   (
        PARTITION BY col_a
        ORDER BY col_b
    ) AS an_aggregate_function
FROM tbl_a
GROUP BY 1, 2
