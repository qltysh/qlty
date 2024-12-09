package pkg1;

class HasIssue {
    public void method() {
        for (int i = 0; i < 10; i++) {
            for (int k = 0; k < 20; i++) {
                System.out.println("Hello");
            }
        }
    }

    public void doSomething(File file, Lock lock) {
        String b = "txt";
        String a = b;
        String c = a;
        b = c;

        file.delete();  // Noncompliant
        lock.tryLock(); // Noncompliant
    }
}
