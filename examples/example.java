public class example {
    // single line comment with a spelling mistake: helllo

    /**
     * Block comment with a spelling mistake: byee
     */

    // variable declarators with spelling mistakes
    short myFavoritNum = 404;
    String myFavoiteString = "Hello, World!";

    // a string with a spelling mistake
    String myStr = "foooooooooooooooood";

    // enum declaration with a spelling mistake
    enum Levvel {
        // enum constant with a spelling mistake
        BEGINER,
        INTERMEDIETE,
        XPERT,
    }

    // interface declaration with a spelling mistake
    public interface Innerexample {
        // method declaration with a spelling mistake
        void doSmething();
    }

    // class declaration with a spelling mistake
    class PointlessClasss {
    }

    public static void main(String[] args) {
        // catch formal parameter spelling mistake
        try {
            int result = 10 / 0;
            System.out.println(result);
        } catch (ArithmeticException uhoooh) {
            System.out.println(uhoooh);
        }
    }

    // method declaration and formal parameter with spelling mistakes
    public String anthrMethod(String smth) {
        // string literal with a spelling mistake
        return "anthr method called with: " + smth;
    }
}
