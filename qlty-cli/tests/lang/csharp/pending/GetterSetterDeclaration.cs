package fields;

public class GetterSetterDeclaration {
  private String field;

  public String getFieldName() {
    return this.field;
  }

  public void setFieldName(String value) {
    this.field = value;
  }

  public static void main(String[] args) {
    GetterSetterDeclaration obj = new GetterSetterDeclaration();
    obj.setFieldName("Hello");
    System.out.println(obj.getFieldName());
  }
}
