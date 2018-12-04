/* 
1) Enqueue transaction that arrives
2) Assign it to a thread
3) Scan for inputs, if passed by value lock the output
3) Dequeue once the evaluated, double spends a locked output before it, or times out
4) Dequeued actions added to state


*/